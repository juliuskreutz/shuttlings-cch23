use actix_web::{
    post,
    web::{self, Buf, Bytes},
    HttpResponse, Responder,
};
use git2::{build::CheckoutBuilder, BranchType};
use tar::Archive;

use crate::ShuttleResult;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(archive_files)
        .service(archive_files_size)
        .service(cookie);
}

#[post("/20/archive_files")]
async fn archive_files(bytes: Bytes) -> ShuttleResult<impl Responder> {
    Ok(Archive::new(bytes.reader()).entries()?.count().to_string())
}

#[post("/20/archive_files_size")]
async fn archive_files_size(bytes: Bytes) -> ShuttleResult<impl Responder> {
    Ok(Archive::new(bytes.reader())
        .entries()?
        .flatten()
        .map(|e| e.size())
        .sum::<u64>()
        .to_string())
}

#[post("/20/cookie")]
async fn cookie(bytes: Bytes) -> ShuttleResult<impl Responder> {
    let dir = tempfile::tempdir()?;

    let mut archive = Archive::new(bytes.reader());
    archive.unpack(&dir)?;

    let repository = git2::Repository::open(&dir)?;
    let branch = repository.find_branch("christmas", BranchType::Local)?;
    let mut current_commit = Some(branch.get().peel_to_commit()?);

    while let Some(commit) = current_commit {
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repository.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;

        for entry in std::fs::read_dir(&dir)?.flatten() {
            let path = entry.path();

            if path.file_name().and_then(|s| s.to_str()) == Some("santa.txt") {
                let content = std::fs::read_to_string(path)?;

                if content.contains("COOKIE") {
                    let author = commit.author().name().unwrap().to_string();
                    let id = commit.id();

                    return Ok(HttpResponse::Ok().body(format!("{author} {id}")));
                }
            }
        }

        current_commit = commit.parent(0).ok();
    }

    Ok(HttpResponse::BadRequest().finish())
}

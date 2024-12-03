use actix_web::{
    post,
    web::{self, Buf, Bytes},
    HttpResponse, Responder,
};
use git2::{build::CheckoutBuilder, BranchType};
use tar::Archive;
use walkdir::WalkDir;

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

    let mut revwalk = repository.revwalk()?;
    revwalk.push(branch.get().target().unwrap())?;

    for oid in revwalk.flatten() {
        let commit = repository.find_commit(oid)?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repository.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;

        for entry in WalkDir::new(&dir)
            .into_iter()
            .flatten()
            .filter(|s| s.file_name().to_str() == Some("santa.txt"))
        {
            let content = std::fs::read_to_string(entry.path())?;

            if content.contains("COOKIE") {
                let author = commit.author().name().unwrap().to_string();
                let id = commit.id();

                return Ok(HttpResponse::Ok().body(format!("{author} {id}")));
            }
        }
    }

    Ok(HttpResponse::BadRequest().finish())
}

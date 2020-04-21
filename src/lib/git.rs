use super::fs::FsRoot;

use dotenv;
use git2::{Cred, Oid, RemoteCallbacks, Repository};
use slog::error;
use std::env;
use std::path::Path;

// const GIT_SSH_KEY: &str = ".ssh/id_rsa";

#[derive(Debug)]
pub struct GitStage {
    pub id: String,
    pub repo: String,
    pub sha: String,
    pub tag: String,
    pub logger: slog::Logger,
}

impl GitStage {
    pub fn new(id: &String, repo: &String, tag: &String, logger: slog::Logger) -> GitStage {
        GitStage {
            id: id.to_owned(),
            repo: repo.to_owned(),
            sha: "".to_owned(),
            tag: tag.to_owned(),
            logger: logger,
        }
    }

    pub fn call(&mut self) -> Option<i32> {
        match self._git_clone_init() {
            Ok(_) => {},
            Err(e) => {
                error!(self.logger, "git clone error: {}", e);

                return Some(400)
            }
        };

        self.sha = match self._git_revparse() {
            Ok(sha) => {
                sha
            },
            Err(e) => {
                error!(self.logger, "git revparse error: {}", e);

                return Some(400)
            }
        };

        match self._git_checkout_commit() {
            Ok(_) => {},
            Err(e) => {
                error!(self.logger, "git checkout error: {}", e);

                return Some(400)
            }
        };

        Some(0)
    }

    fn _git_checkout_commit(&self) -> Result<(), git2::Error> {
        let repo = Repository::open(&FsRoot::call(&self.id))?;

        let oid = Oid::from_str(&self.sha).unwrap();
        let commit = repo.find_commit(oid).unwrap();

        let _branch = repo.branch(
            &self.sha,
            &commit,
            false,
        );

        let obj = repo.revparse_single(&("refs/heads/".to_owned() + &self.sha)).unwrap();

        repo.checkout_tree(&obj, None)?;

        repo.set_head(&("refs/heads/".to_owned() + &self.sha))?;

        Ok(())
    }

    fn _git_clone_init(&self) -> Result<(), git2::Error> {
        // git ssh callback
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(&format!("{}/{}", env::var("HOME").unwrap(), dotenv::var("GIT_SSH_KEY").unwrap())),
                None,
            )
        });


        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder.clone(
            &self.repo,
            Path::new(&FsRoot::call(&self.id)),
        )?;

        Ok(())
    }

    fn _git_revparse(&self) ->  Result<String, git2::Error> {
        let repo = Repository::open(&FsRoot::call(&self.id))?;

        let revspec = repo.revparse(&self.tag)?;

        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            // println!("single {}", revspec.from().unwrap().id());

            return Ok(revspec.from().unwrap().id().to_string())
        } else if revspec.mode().contains(git2::RevparseMode::RANGE) {
            let to = revspec.to().unwrap();
            let from = revspec.from().unwrap();
            // println!("range {}", to.id());

            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                let _base = repo.merge_base(from.id(), to.id())?;
                // println!("merge {}", base);
            }

            // println!("^{}", from.id());

            return Ok(from.id().to_string())
        } else {
            return Err(git2::Error::from_str("invalid results from revparse"));
        }
    }
}

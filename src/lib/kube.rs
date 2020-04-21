use super::kube_files_apply::KubeFilesApply;
use super::kube_files_rewriter::KubeFilesRewriter;
use super::kube_resource::KubeResourceResolve;

#[derive(Debug)]
pub struct KubeStage {
    pub id: String,
    pub resource_file: String,
    pub resource_key: String,
    pub image_tag: String,
    pub logger: slog::Logger,
}

impl KubeStage {
    pub fn new(id: &String, resource_path: &String, image_tag: &String, logger: slog::Logger) -> KubeStage {
        let resource_vec: Vec<_> = resource_path.split(":").collect();
        let resource_file = KubeResourceResolve::call(id, resource_path);

        KubeStage {
            id: id.to_owned(),
            resource_file: resource_file,
            resource_key: resource_vec[1].to_owned(),
            image_tag: image_tag.to_owned(),
            logger: logger,
        }
    }

    pub fn call(&self) -> Option<i32> {
        let files_rewriter = KubeFilesRewriter::new(
            &self.id,
            &self.resource_file,
            &self.resource_key,
            &self.image_tag,
        );

        let files_latest = match files_rewriter.call() {
            None => {
                return Some(400)
            },
            Some(files) => {
                files
            }
        };

        // kubectl apply new resources (e.g. deployments, sts)

        let files_apply = KubeFilesApply::new(
            &self.id,
            &self.resource_file,
            &self.resource_key,
            files_latest,
        );

        match files_apply.call(self.logger.clone()) {
            None => {
                return Some(400)
            },
            Some(_) => {}
        };

        Some(0)
    }

}

use parking_lot::RwLock;
use std::sync::Arc;

use crate::rendering;

#[derive(Debug)]
pub struct Camera {
    pub camera: Arc<RwLock<rendering::camera::Camera>>,
}

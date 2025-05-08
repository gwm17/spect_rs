use uuid::Uuid;

pub struct CutSpec {
    id: Uuid,
    x_variable: String,
    y_variable: Option<String>,
}

pub trait Cut {
    fn is_inside(value_x: f32, value_y: Option<f32>) -> Result<(), CutError>
}

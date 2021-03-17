use crate::output_destination::OutputDestination;

pub struct Selection {
    pub text: String,
    pub uppercased: bool,
    pub output_destination: OutputDestination,
}

#[derive(Debug, Clone)]
pub struct MappingError;

// type alias for mapping result.
pub type MapResult<T> = std::result::Result<T, MappingError>;

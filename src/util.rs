/// Split a "owner/name" model identifier into separate parts.
pub fn split_model_id(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or(("", id))
}

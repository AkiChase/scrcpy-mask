use serde::Serializer;

/// Serialize f32 rounded to 3 decimal places in f64 to avoid
/// IEEE 754 artifacts like 1.4 becoming 1.399999976158142.
pub fn serialize_f32_3dp<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
    let rounded = (*value as f64 * 1000.0).round() / 1000.0;
    serializer.serialize_f64(rounded)
}

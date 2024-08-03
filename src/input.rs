use cbor_diag::DataItem;

/// see https://www.rfc-editor.org/rfc/rfc8610#appendix-G
fn parse_input(edn: &[u8]) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse() {
        DataItem::Map {}
    }
}

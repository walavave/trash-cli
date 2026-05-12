use crate::error::{Error, Result};

pub fn parse_indexes(input: &str, len: usize) -> Result<Vec<usize>> {
    let mut indexes = Vec::new();

    for raw_part in input.split(',') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((first, last)) = part.split_once('-') {
            let first = parse_index(first.trim(), len)?;
            let last = parse_index(last.trim(), len)?;
            if first > last {
                return Err(Error::InvalidSelection(format!("invalid range: {part}")));
            }

            for index in first..=last {
                indexes.push(index);
            }
        } else {
            indexes.push(parse_index(part, len)?);
        }
    }

    if indexes.is_empty() {
        return Err(Error::InvalidSelection("empty selection".to_string()));
    }

    Ok(indexes)
}

fn parse_index(text: &str, len: usize) -> Result<usize> {
    let index: usize = text
        .parse()
        .map_err(|_| Error::InvalidSelection(format!("not an index: {text}")))?;
    if index >= len {
        return Err(Error::InvalidSelection(format!(
            "out of range 0..{}: {index}",
            len.saturating_sub(1)
        )));
    }
    Ok(index)
}

use ciborium::ser::into_writer;
use ciborium::de::from_reader;
use lz4::EncoderBuilder;
use lz4::Decoder;
use sha2::{Sha256, Digest};
use std::error::Error;
use std::io::{Cursor, Read, Write};
use crate::models::event::Event;
use crate::models::event::SerializedEvent;

/// Serializa a CBOR, comprime con lz4 y calcula hash
pub fn serialize_event(event: &Event) -> Result<SerializedEvent, Box<dyn Error>> {
    
    let mut cbor_bytes: Vec<u8> = Vec::new();
    into_writer(event, &mut cbor_bytes)?;

    // Comprimir con LZ4 (stream)
    let mut encoder: lz4::Encoder<Vec<u8>> = EncoderBuilder::new().level(4).build(Vec::new())?;
    encoder.write_all(&cbor_bytes)?;
    let (compressed, result): (Vec<u8>, Result<(), std::io::Error>) = encoder.finish();
    result?;

    // Hash SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&compressed);
    let hash = hasher.finalize().to_vec();

    Ok(SerializedEvent { payload: compressed, hash })
}

/// Deserializa payload comprimido, validando hash
pub fn deserialize_event(payload: &[u8], expected_hash: &[u8]) -> Result<Event, Box<dyn Error>> {
    // Validar hash
    let mut hasher = Sha256::new();
    hasher.update(payload);
    if hasher.finalize().as_slice() != expected_hash {
        return Err("Hash mismatch: payload corrupto".into());
    }

    // Descomprimir LZ4
    let mut decoder = Decoder::new(Cursor::new(payload))?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // Deserializar CBOR
    let event: Event = from_reader(&decompressed[..])?;
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let ev = Event {
            user_id: "user123".into(),
            action: "click".into(),
            value: 42,
        };

        let serialized = serialize_event(&ev).unwrap();
        let deserialized = deserialize_event(&serialized.payload, &serialized.hash).unwrap();

        assert_eq!(deserialized.user_id, ev.user_id);
        assert_eq!(deserialized.action, ev.action);
        assert_eq!(deserialized.value, ev.value);
    }
}

use olm_rs::inbound_group_session::OlmInboundGroupSession;

/** Implements Decryption of messages encrypted with the MEGOLM Algorithm used in Matrix */

struct OLMDecoder {
    group_session: OlmInboundGroupSession,
}

impl OLMDecoder {
    pub fn new(session_key: &str) -> OLMDecoder {
        OLMDecoder {
            group_session: OlmInboundGroupSession::new(session_key).expect("Unable to initialize OLM!")
        }
    }

    pub fn decode(&self, msg: String) -> String {
        self.group_session.decrypt(msg).expect("Unable to decrypt!").0
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::OLMDecoder;

    #[test]
    fn it_works() {
        let decoder = OLMDecoder::new("AgAAAADXhns1QbrnL4YHuAdBYPkZyAhXTPl1tlKcbGw/5zYVsNYVOKRhGKGbCyDBu7iYjSYsvXElDl1Fg4vBKMuSIuidh18417lNphzu80BOFujtnQ8n3Q7tx2jBZGBaF9NHgVWJJSNj+o1/Nx91l8ks5z8kWWYUXljyNDDOMiyB26joiTV/kHd8AEaQ1aoxoQZ2l5UfUVBN2jQMYyNkubsxm+hdeJ8hUfH+l1HzGcZsCqN7ZVajgkOuucfj3pgOUUL4KwVupHny7g3sCa5ZTaFFb1SWUcRPMZb5qq/IVUtBFU42DQ");

        let result = decoder.decode("AwgAEjAoR9/W8oDdqj8wrw7gsy1AXn6s5TTh8R45L+mm+7hcnERxAW13pgyxjyXeNnrogJDlTivQjo46u6iwXJEONlcpiQOAmyo8faORwxTjntKeFi22VGqrcdr+z00UfHc0ceek6h2QPq2T52AwhyYq6JTkUp4RwQHdnAM".to_string());

        assert_eq!(result, "Hallo Jens! (Das ist Nachricht 1)");
    }
}

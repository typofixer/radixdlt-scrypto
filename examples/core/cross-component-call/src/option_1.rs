use scrypto::prelude::*;

import! {
r#"
{
    "package": "01bda8686d6c2fa45dce04fac71a09b54efbc8028c23aac74bc00e",
    "name": "Airdrop",
    "functions": [
        {
            "name": "new",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::core::Component",
                "generics": []
            }
        }
    ],
    "methods": [
        {
            "name": "free_token",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        }
    ]
}
"#
}

blueprint! {
    struct Proxy1 {
        airdrop: Airdrop
    }

    impl Proxy1 {
        pub fn new() -> Component {
            Self {
                airdrop: Airdrop::new().into()
            }
            .instantiate()
        }

        pub fn free_token(&self) -> Bucket {
            self.airdrop.free_token()
        }
    }
}

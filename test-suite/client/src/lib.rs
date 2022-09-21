use tonic_web_wasm_client::Client;

use self::proto::echo_client::EchoClient;

pub mod proto {
    tonic::include_proto!("echo");
}

impl Default for EchoClient<Client> {
    fn default() -> Self {
        let base_url = "https://localhost:50051".to_string();
        let wasm_client = Client::new(base_url);

        EchoClient::new(wasm_client)
    }
}

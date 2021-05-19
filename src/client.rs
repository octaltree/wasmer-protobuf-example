use hello_world::{greeter_client::GreeterClient, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GreeterClient::connect("http://[::1]:50051").await?;

    for _ in 0..300_000 {
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into()
        });

        let mut c2 = client.clone();
        let response = tokio::spawn(async move { c2.say_hello(request).await });

        println!("RESPONSE={:?}", response);
    }

    Ok(())
}

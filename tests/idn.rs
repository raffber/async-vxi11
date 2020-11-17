use async_vxi11::CoreClient;
use std::net::IpAddr;

#[tokio::test]
async fn check_idn_read() {
    let addr: IpAddr = "192.168.0.60".parse().unwrap();
    let mut client = CoreClient::connect(addr).await.unwrap();
    client.device_write("*IDN?\n".as_bytes().to_vec()).await.unwrap();
    let result = client.device_read().await.unwrap();
    let ret = String::from_utf8(result).unwrap();
    println!("{}", ret);
}
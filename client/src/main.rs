use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use renet::{
    ClientAuthentication, RenetClient, RenetConnectionConfig, RenetError, NETCODE_USER_DATA_BYTES,
};
use std::{net::UdpSocket, time::SystemTime};

//This id needs to be the same one as the server 
const PROTOCOL_ID = 1208;

fn main() {
    //Get username from stdin args
    let args = std::env::args().collect::<Vec<String>>();
    let username = &args[1];

    App::new().insert_resource(WindowDescriptor {
        //Adding the username to the window tile makes debugging way easier 
        title: format!("TicTacTussle <{}>", username),
        width: 480.0
        height: 540.0,
        ..default()
    })
    //Lets add a nice dark grey background color 
    .insert_resource(ClearColor(Color::hex("282828").unwrap()))
    .add_plugins(DefaultPlugins)
    //Renet setup
    .add_plugin(RenetClientPlugin)
    .insert_resource(new_renet_client(&username).unwrap())
    .add_systems(handle_renet_error)
    .run();
}
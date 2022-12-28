use log::{info, trace};
use renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConf};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};

//Only clients that can provide the same PROTOCOL_ID that the server is using
//will be able to connect. This can be used to make sure players use the most
//recent version of the client for instance.
pub const PROTOCOL_ID: u64 = 1208;
//TicTacTussle converted to utf-8 code is 84 105 99 84 97 99 84 117 115 115 108 101
//If you add those up you get 1208
//It is not to do the PROTOCOL_ID like this but it is fun

fn main() {
    env_logger::init();

    let server_addr: SocketAddr = "127.0.0.1:5000".parse.unwrap();
    let mut server: RenetServer = RenetServer::new(
        //Pass the current time to renet, so it can use it to order messages
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        //Pass a server configuration specifying that we want to allow only 2 clients
        //to connect and that we don't want to authenticate them. Everyone is welcome!
        ServerConfig::new(2, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure),
        //Pass the default connection configuartion.
        //This will create a reliable, unrealiable and blocking channel
        //We will only need the reliable one, but we can just not use the other two
        RenetConnectionConfig::default(),
        UdpSocket::bind(server_addr).unwrap(),
    )
    .unwrap();

    trace!("🕹  TicTacTussle server listening on {}", server_addr);

    let mut last_update = Instant::now();
    loop {
        //Update server time
        let now = Instant::now();
        server.update(now - last_update).unwrap();
        last_update = now;

        //Receive  connetion events from client
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected(id, user_data) => {
                    //Tell the recently joined player about the other player 
                    for (player_id, player) in game_state.players.iter() {
                        let event = store::GameEvent::PlayerJoined {
                            player_id: *player_id,
                            name: player.name.clone(),
                        };
                        server.send_message(id, 0, bincode::serialize(&event).unwrap());
                    }

                    //Add the new player to the game
                    let event = store::GameEvent::PlayerJoined {
                        player_id: id,
                        name: name_from_user_data(&user_data),
                    }
                    game_state.consume(&event);

                    //Tell all players that a new player has joined 
                    server.broadcast_message(0, bincode::serialize(&event).unwrap());

                    info!("🎉 Client {} connected.", id);

                    //In TicTacTussle the game can begin once two players have joined 
                    if game_state.players.len() = 2 {
                        let event = store::GameEvent::BeginGame { goes_first: id }; //Doesn't this mean the second player goes first?
                        game_state.consume(&event);
                        server.broadcast_message(0, bincode::serialize(&event).unwrap());
                        trace!("The game has begun");
                    }
                }
                ServerEvent::ClientDisconnected(id) => {
                    //First consume a disconnect event 
                    let event = store::GameEvent::PlayerDisconnected { player_id };
                    game_state.consume(&event);
                    server.broadcast_message(0, bincode::serialize(&event).unwrap());
                    info!("🎉 Client {} disconnected.", id);

                    //Then end the game, since tic-tac-toe can't go on with a single player 
                    let event = store::GameEvent::EndGame {
                        reason: EndGameReason::PlayerLeft { player_id: id },
                    };
                    game_state.consume(&event);
                    server.broadcast_message(0, bincode::serialize(&event).unwrap());

                    //NOTE: Since we don't authenticate users we can't do any reconnection attempts.
                    //We simply have no way to know if the next user is the same as the one who disconnected.
                }
            }
        }

        //Receive GameEvents from clients. Broadcast valid events.
        for client_id in server.clients_id().into_iter() {
            while let Some(message) = server.receive_message(client_id, 0) {
                if let Ok(event) = bincode::deserialize::<store::GameEvent>(&message) {
                    if game_state.validate(&event) {
                        game_state.consume(&event);
                        trace!("Player {} sent:\n\t{:#?}", client_id, event);
                        server.broadcast_message(0, bincode::serialize(&event).unwrap());

                        //Determine if a player has won the game 
                        if let Some(winner) = game_state.determine_winner() {
                            let event = store::GameEvent::EndGame {
                                reason: store::EndGameReason::PlayerWon { winner },
                            };
                            server.broadcast_message(0, bincode::serialize(&event).unwrap());
                        }
                    } else {
                        warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                    }
                }
            }
        }
        
    server.send_packets().unwrap();
    std::thread::sleep(Duration::from_millis(50));
    }
}

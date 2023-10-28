use bevy::prelude::{Events, Mut, ResMut, Time, World};
use tracing::{debug, trace};

use lightyear_shared::connection::events::IterEntitySpawnEvent;
use lightyear_shared::{
    ConnectEvent, DisconnectEvent, EntitySpawnEvent, MessageProtocol, Protocol,
};

use crate::Client;

pub(crate) fn receive<P: Protocol>(world: &mut World) {
    trace!("Receive server packets");
    world.resource_scope(|world, mut client: Mut<Client<P>>| {
        let time = world.get_resource::<Time>().unwrap();

        // update client state, send keep-alives, receive packets from io
        client.update(time.elapsed().as_secs_f64()).unwrap();
        // buffer packets into message managers
        client.recv_packets().unwrap();
        // receive packets from message managers
        let mut events = client.receive(world);
        if !events.is_empty() {
            dbg!(&events.spawns);
            // panic!();

            if events.has_connection() {
                let mut connect_event_writer =
                    world.get_resource_mut::<Events<ConnectEvent>>().unwrap();
                debug!("Client connected event");
                connect_event_writer.send(ConnectEvent::new(()));
            }

            if events.has_disconnection() {
                let mut disconnect_event_writer =
                    world.get_resource_mut::<Events<DisconnectEvent>>().unwrap();
                debug!("Client disconnected event");
                disconnect_event_writer.send(DisconnectEvent::new(()));
            }

            // Message Events
            P::Message::push_message_events(world, &mut events);

            // Spawn entity event
            if events.has_entity_spawn() {
                let mut entity_spawn_event_writer = world
                    .get_resource_mut::<Events<EntitySpawnEvent>>()
                    .unwrap();
                for (entity, _) in events.into_iter_entity_spawn() {
                    entity_spawn_event_writer.send(EntitySpawnEvent::new(entity, ()));
                }
            }
        }
    });
}

pub(crate) fn send<P: Protocol>(mut client: ResMut<Client<P>>) {
    trace!("Send packets to server");
    // send buffered packets to io
    client.send_packets().unwrap();
}
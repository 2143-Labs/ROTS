use bevy::prelude::*;

use crate::event::{client::SomeoneCast, server::Cast, NetEntId};

use super::{AnimationTimer, CastNetId, CastPointTimer, DoCast};

//TODOs:
// - [ ] player can cancel casting their stuff
// - [ ] server can stop a player from casting
//
///go until the cast point and then do the actual effect
pub fn tick_casts(
    mut casting_units: Query<(
        Entity,
        &NetEntId,
        &mut CastPointTimer,
        &mut AnimationTimer,
        &Cast,
        Option<&CastNetId>,
    )>,
    mut commands: Commands,
    mut do_cast: EventWriter<DoCast>,
    time: Res<Time<Virtual>>,
) {
    for (ent, net_ent_id, mut cast_timer, mut anim_timer, cast, cast_net_id) in &mut casting_units {
        cast_timer.0.tick(time.delta());
        anim_timer.0.tick(time.delta());

        if cast_timer.0.finished() {
            if let Some(caster) = cast_net_id {
                do_cast.send(DoCast(SomeoneCast {
                    caster_id: *net_ent_id,
                    cast_id: caster.0,
                    cast: cast.clone(),
                }));
            } else {
                warn!("server never sent us the casting data, not sure if we should cast this");
            }

            cast_timer.0.reset();
            cast_timer.0.pause();
        }

        if anim_timer.0.finished() {
            commands
                .entity(ent)
                .remove::<AnimationTimer>()
                .remove::<CastPointTimer>()
                .remove::<Cast>()
                .remove::<CastNetId>();
        }
    }
}

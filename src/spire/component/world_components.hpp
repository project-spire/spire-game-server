#pragma once

#include <spire/core/units.hpp>

namespace spire {
struct WorldPortal {
    u32 id;
    u32 target_world_id;
    u32 target_room_id;
};

struct RoomPortal {
    u32 id;
    u32 target_room_id;
};
}
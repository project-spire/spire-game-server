#pragma once

#include <spire/core/units.hpp>

namespace spire {
struct RoomTransfer {
    enum class State {
        ClientRequested,
        ClientLoading,
        ClientReady
    };

    State state;
    u32 origin_room_id;
    u32 portal_id;
};
}
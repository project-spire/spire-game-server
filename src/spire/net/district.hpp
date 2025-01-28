#pragma once

#include <spire/net/room.hpp>

namespace spire::net {
class District {
public:
    void broadcast_message(std::shared_ptr<OutMessage> message);

private:
    std::unordered_map<u32, std::shared_ptr<Room>> _rooms {};
};
}
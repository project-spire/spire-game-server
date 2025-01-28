#include <spire/net/district.hpp>

namespace spire::net {
void District::broadcast_message(std::shared_ptr<OutMessage> message) {
    for (const auto& room : _rooms | std::views::values)
        room->broadcast_message_deferred(message);
}
}

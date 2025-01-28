#include <spire/net/client.hpp>
#include <spire/net/room.hpp>
#include <spire/msg/base_message.pb.h>

namespace spire::net {
Client::Client(
    const u64 id,
    boost::asio::ip::tcp::socket&& socket,
    const std::shared_ptr<Room>& current_room,
    std::function<void(std::shared_ptr<Client>)>&& on_stop)
    : _strand {make_strand(socket.get_executor())},
    _heartbeater {
          socket.get_executor(),
          [self = shared_from_this()] {
              msg::Heartbeat heartbeat;
              msg::BaseMessage base;
              base.set_allocated_heartbeat(&heartbeat);

              self->send(std::make_unique<OutMessage>(base));
          },
          [self = shared_from_this()] {
              self->stop(StopCode::DeadHeartbeat);
          }},
    _connection {
        std::move(socket),
        [self = shared_from_this()](const Connection::CloseCode code) {
            self->stop(code == Connection::CloseCode::Normal ? StopCode::Normal : StopCode::ConnectionError);
        },
        [self = shared_from_this()](std::vector<std::byte>&& data) {
            if (self->_is_authenticated) {
                self->_heartbeater.pulse();
            }

            if (const auto room {self->_current_room.load().lock()}; room) {
                room->post_message(
                    std::make_unique<InMessage>(self->shared_from_this(), std::move(data)));
            }
        }},
    _on_stop {std::move(on_stop)},
    _current_room {current_room},
    _character_id {id} {}

Client::~Client() {
    stop(StopCode::Normal);
}

void Client::start() {
    if (_is_running.exchange(true)) return;

    _connection.open();
}

void Client::stop(const StopCode code) {
    if (!_is_running.exchange(false)) return;

    if (code != StopCode::Normal) {
        //TODO: Log
    }

    _connection.close(Connection::CloseCode::Normal);

    _on_stop(shared_from_this());
}

void Client::send(std::unique_ptr<OutMessage> message) {
    _connection.send(std::move(message));
}

void Client::send(std::shared_ptr<OutMessage> message) {
    _connection.send(std::move(message));
}

void Client::authenticate(const u32 account_id, const u32 character_id) {
    _is_authenticated = true;

    _account_id = account_id;
    _character_id = character_id;
}
}

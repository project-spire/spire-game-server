#pragma once

#include <spire/net/connection.hpp>
#include <spire/net/heartbeater.hpp>
#include <spire/net/message.hpp>

namespace spire::net {
class Room;

class Client final : public std::enable_shared_from_this<Client>, boost::noncopyable {
public:
    enum class StopCode {
        Normal,
        InvalidInMessage,
        ConnectionError,
        DeadHeartbeat,
        AuthenticationError
    };

    Client(
        u64 id,
        boost::asio::ip::tcp::socket&& socket,
        const std::shared_ptr<Room>& current_room,
        std::function<void(std::shared_ptr<Client>)>&& on_stop);
    ~Client();

    void start();
    void stop(StopCode code);

    void send(std::unique_ptr<OutMessage> message);
    void send(std::shared_ptr<OutMessage> message);

    void authenticate(u32 account_id, u32 character_id);

    u64 id() const { return _character_id; }
    u64 account_id() const { return _account_id; }
    u64 character_id() const { return _character_id; }
    milliseconds ping() const { return _ping.load(); }
    std::atomic<std::weak_ptr<Room>>& current_room() { return _current_room; }

private:
    boost::asio::strand<boost::asio::any_io_executor> _strand;
    Heartbeater _heartbeater;
    Connection _connection;
    std::atomic<milliseconds> _ping {};

    std::atomic<bool> _is_running {false};
    bool _is_authenticated {false};
    std::function<void(std::shared_ptr<Client>)> _on_stop;

    std::atomic<std::weak_ptr<Room>> _current_room;

    u64 _account_id {};
    u64 _character_id;
};
}
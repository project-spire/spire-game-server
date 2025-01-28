#include <spdlog/spdlog.h>
#include <spire/core/settings.hpp>
#include <spire/net/server.hpp>
#include <spire/room/waiting_room.hpp>

namespace spire::net {
Server::Server(boost::asio::any_io_executor&& io_executor)
    : _io_executor {std::move(io_executor)},
    _io_strand {make_strand(_io_executor)},
    _acceptor {
        make_strand(_io_executor),
        boost::asio::ip::tcp::endpoint {boost::asio::ip::tcp::v4(), Settings::listen_port()}},
    _waiting_room {std::make_shared<WaitingRoom>(0, _io_executor, _work_executor)} {
    _acceptor.set_option(boost::asio::socket_base::reuse_address(true));
    _acceptor.listen(Settings::listen_backlog());
}

Server::~Server() {
    stop();
}

void Server::start() {
    if (_is_running.exchange(true)) return;

    // Spawn acceptor loop
    co_spawn(_io_executor, [this] -> boost::asio::awaitable<void> {
        while (_is_running) {
            auto [ec, socket] = co_await _acceptor.async_accept(boost::asio::as_tuple(boost::asio::use_awaitable));
            if (ec) {
                if (ec == boost::asio::error::operation_aborted) continue;

                spdlog::warn("Error accepting socket");
                continue;
            }

            socket.set_option(boost::asio::ip::tcp::no_delay(true), ec);
            if (ec) {
                spdlog::warn("Error setting socket option");
                continue;
            }

            create_client_deferred(std::move(socket));
        }
    }, boost::asio::detached);

    _waiting_room->start();
    for (const auto& room : _rooms | std::views::values)
        room->start();
}

void Server::stop() {
    if (!_is_running.exchange(false)) return;

    {
        boost::system::error_code ec;
        _acceptor.close(ec);
    }

    std::promise<void> cleanup_promise {};
    const auto cleanup_future {cleanup_promise.get_future()};

    dispatch(_io_strand, [this, cleanup_promise = std::move(cleanup_promise)] mutable {
        spdlog::info("Server cleanup...");

        for (const auto& client : _clients | std::views::values)
            client->stop(Client::StopCode::Normal);

        _waiting_room->stop();
        for (const auto& room : _rooms | std::views::values)
            room->stop();

        cleanup_promise.set_value();
    });
    cleanup_future.wait();

    spdlog::info("Server cleanup done.");
}

void Server::create_client_deferred(boost::asio::ip::tcp::socket&& socket) {
    static std::atomic<u64> temp_client_id_generator {0};

    auto new_client {std::make_shared<Client>(
        ++temp_client_id_generator,
        std::move(socket),
        _waiting_room,
        [this](std::shared_ptr<Client> client) mutable {
            remove_client_deferred(std::move(client));
        })};

    dispatch(_io_strand, [this, new_client = std::move(new_client)] mutable {
        _clients[new_client->id()] = new_client;
        _waiting_room->add_client_deferred(std::move(new_client));
    });
}

void Server::remove_client_deferred(std::shared_ptr<Client> client) {
    client->stop(Client::StopCode::Normal);

    dispatch(_io_strand, [this, client = std::move(client)] {
        if (const auto current_room {client->current_room().load().lock()}) {
            current_room->remove_client_deferred(client);
        }
        _clients.erase(client->id());
    });
}

void Server::transfer_client_deferred(std::shared_ptr<Client> client, const u32 target_room_id) {
    dispatch(_io_strand, [this, client = std::move(client), target_room_id] {
        if (_rooms.contains(target_room_id)) {
            spdlog::warn("No room with target id {}.", target_room_id);
            return;
        }

        if (const auto current_room {client->current_room().load().lock()}) {
            current_room->remove_client_deferred(client);
        }
        _rooms.at(target_room_id)->add_client_deferred(client);
    });
}

void Server::add_room_deferred(std::shared_ptr<Room> room) {
    dispatch(_io_strand, [this, room = std::move(room)] mutable {
        _rooms.insert_or_assign(room->id(), std::move(room));
    });
}

void Server::remove_room_deferred(const u32 room_id) {
    dispatch(_io_strand, [this, room_id] {
        _rooms.erase(room_id);
    });
}
}

#include <spdlog/spdlog.h>
#include <spire/handler/auth_handler.hpp>
#include <spire/handler/net_handler.hpp>
#include <spire/room/waiting_room.hpp>

namespace spire {
WaitingRoom::WaitingRoom(boost::asio::any_io_executor& io_executor)
    : Room {0, io_executor} {
    _handler_controller.add_handler(NetHandler::make());
    _handler_controller.add_handler(AuthHandler::make());
}

void WaitingRoom::on_client_entered(const std::shared_ptr<net::TcpClient>& client) {
    client->start();
}
}

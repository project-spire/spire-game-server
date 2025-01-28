#include <boost/asio/socket_base.hpp>
#include <spire/core/settings.hpp>
#include <yaml-cpp/yaml.h>

#include <cstdlib>
#include <fstream>
#include <stdexcept>

namespace spire {
std::string read_file_line(std::string_view path) {
    std::ifstream file {path.data()};
    if (!file.is_open())
        throw std::invalid_argument("Invalid file path");

    std::string value;
    std::getline(file, value);
    return value;
}

void Settings::init() {
    YAML::Node settings {YAML::LoadFile(SPIRE_SETTINGS_FILE)};

    _listen_port = std::stoi(std::getenv("SPIRE_GAME_LISTEN_PORT"));
    _listen_backlog = settings["listen_backlog"]
        ? settings["listen_backlog"].as<u16>()
        : boost::asio::socket_base::max_listen_connections;

    _auth_jwt_key = read_file_line(std::getenv("SPIRE_AUTH_JWT_KEY_FILE"));

    _db_host = std::getenv("SPIRE_DB_HOST");
    _db_port = std::stoi(std::getenv("SPIRE_DB_PORT"));
    _db_name = std::getenv("SPIRE_DB_NAME");
    _db_user = std::getenv("SPIRE_DB_USER");
    _db_password = read_file_line(std::getenv("SPIRE_DB_PASSWORD_FILE"));
}
}

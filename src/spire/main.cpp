#include <boost/mysql/src.hpp>

#include <boost/mysql.hpp>
#include <spdlog/spdlog.h>
#include <spire/core/settings.hpp>
#include <spire/net/server.hpp>

int main() {
    using namespace spire;

    Settings::init();

#ifndef NDEBUG
    spdlog::set_level(spdlog::level::debug);
#endif
    spdlog::info("spdlog log level: {}", to_string_view(spdlog::get_level()));

    boost::asio::thread_pool io_threads {std::thread::hardware_concurrency() - 1};
    boost::asio::signal_set signals {io_threads.get_executor(), SIGINT, SIGTERM};
    net::Server server {io_threads.get_executor()};

    boost::mysql::pool_params db_params {};
    db_params.server_address.emplace_host_and_port(Settings::db_host().data(), Settings::db_port());
    db_params.database = Settings::db_name();
    db_params.username = Settings::db_user();
    db_params.password = Settings::db_password();
    db_params.thread_safe = true;
    db_params.initial_size = std::thread::hardware_concurrency();
    db_params.multi_queries = true;

    boost::mysql::connection_pool db_pool {io_threads.get_executor(), std::move(db_params)};

    signals.async_wait([&io_threads, &server, &db_pool](boost::system::error_code, int) {
        server.stop();
        db_pool.cancel();
        io_threads.stop();
    });

    db_pool.async_run(boost::asio::detached);
    server.start();

    io_threads.attach();
    io_threads.join();

    return EXIT_SUCCESS;
}

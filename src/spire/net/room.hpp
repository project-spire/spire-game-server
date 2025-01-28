#pragma once

#include <entt/entt.hpp>
#include <spire/container/concurrent_queue.hpp>
#include <spire/net/client.hpp>
#include <spire/handler/handler_controller.hpp>
#include <taskflow/taskflow.hpp>

namespace spire::net {
class Room : public std::enable_shared_from_this<Room>, boost::noncopyable {
public:
    Room(
        u32 id,
        boost::asio::any_io_executor& io_executor,
        tf::Executor& work_executor);
    virtual ~Room();

    void start();
    void stop();

    void add_client_deferred(std::shared_ptr<Client> client);
    void remove_client_deferred(std::shared_ptr<Client> client);

    void post_task(std::function<void()>&& task);
    void post_message(std::unique_ptr<InMessage> message);
    void broadcast_message_deferred(std::shared_ptr<OutMessage> message);

    u32 id() const { return _id; }

private:
    virtual void on_client_entered(const std::shared_ptr<Client>& client) {}
    virtual void on_client_left(const std::shared_ptr<Client>& client) {}

    void update(time_point<steady_clock> last_update_time);
    virtual void compose_systems(tf::Taskflow& taskflow, time_point<steady_clock> now, f32 dt) {}

protected:
    HandlerController _handler_controller {};
    entt::registry _registry {};

private:
    const u32 _id;

    std::atomic<bool> _is_running {false};
    boost::asio::any_io_executor& _io_executor;
    tf::Executor& _work_executor;

    ConcurrentQueue<std::function<void()>> _tasks {};
    ConcurrentQueue<std::unique_ptr<InMessage>> _messages {};

    std::unordered_map<u64, std::shared_ptr<Client>> _clients {};
};
}
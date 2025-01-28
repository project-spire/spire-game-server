#pragma once

#include <spire/handler/types.hpp>
#include <spire/net/message.hpp>

namespace spire {
class HandlerController {
public:
    HandlerController() = default;
    ~HandlerController() = default;
    HandlerController(HandlerController&& other) noexcept;
    HandlerController& operator=(HandlerController&& other) noexcept;

    void add_handler(HandlerFunction&& handler);
    void handle_message(std::unique_ptr<net::InMessage> message) const;

private:
    std::list<HandlerFunction> _handlers {};
};


inline HandlerController::HandlerController(HandlerController &&other) noexcept
    : _handlers {std::move(other._handlers)} {}

inline HandlerController & HandlerController::operator=(HandlerController &&other) noexcept {
    if (this != &other) {
        _handlers = std::move(other._handlers);
    }
    return *this;
}

inline void HandlerController::add_handler(HandlerFunction&& handler) {
    _handlers.push_back(std::move(handler));
}

inline void HandlerController::handle_message(std::unique_ptr<net::InMessage> message) const {
    msg::BaseMessage base {};
    if (!base.ParseFromArray(message->data(), static_cast<int>(message->size()))) {
        message->client()->stop(net::Client::StopCode::InvalidInMessage);
    }

    for (const auto& handler : _handlers) {
        const auto result {handler(message->client(), base)};
        if (result == HandlerResult::Break || result == HandlerResult::Error) break;
    }
}
}

#pragma once

#include <entt/entt.hpp>
#include <spire/component/physics_components.hpp>
#include <spire/core/units.hpp>


namespace spire::physics {
struct PhysicsConfig {
    Acceleration gravity {9.8};
};


class PhysicsSystem final {
public:
    static void update(entt::registry& registry, f32 dt);
};

inline void PhysicsSystem::update(entt::registry& registry, const f32 dt) {
    registry.view<Transform, DynamicPhysics>().each([dt](auto& transform, auto& dynamic) {
        transform.position += dt * dynamic.velocity;
    });
}
}

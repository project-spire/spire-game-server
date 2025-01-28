#pragma once

#include <glm/vec3.hpp>
#include <spire/core/units.hpp>

namespace spire {
struct Transform {
    glm::vec3 position;
    radian rotation;
};

struct StaticPhysics {

};

struct KinematicPhysics {

};

struct DynamicPhysics {
    glm::vec3 velocity;
    Acceleration acceleration;
};
}
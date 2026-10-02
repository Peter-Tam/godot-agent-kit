#pragma once
#include "editor_context.hpp"

namespace gak {
const char *opening_context(const Session &session, const std::string &request,
        const OpeningEffective &effective, OpeningContext &out,
        const OpeningContext *retained = nullptr, bool after_open = false);
} // namespace gak

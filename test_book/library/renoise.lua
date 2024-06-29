---@meta

---@class renoise
renoise = {}

---Renoise application version.
---@type string
renoise.RENOISE_VERSION = "Major.Minor.Revision[AlphaBetaRcVersion][Demo]"

---version number (e.g. 1.0 -> 1.1).
---and classes which do not break existing scripts, will increase only the
---All other backwards compatible changes, like new functionality, new fun
---will increase the internal API's major version number (e.g. from 1.4 ->
---Currently 6.1). 
---@type number
renoise.API_VERSION = 6.1

---Global access to the Renoise Application.
---@return renoise.Application
function renoise.app() end
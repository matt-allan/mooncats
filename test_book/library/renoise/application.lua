---@meta

---The Renoise application.
---@class renoise.Application
---
---**READ-ONLY** Access to the application's full log filename and path. Will
---already be opened for writing, but you nevertheless should be able to read
---from it.
---@field log_filename string The path to the log file used by Renoise.
renoise.Application = {}

---Shows an info message dialog to the user.
---@param message string an informative message
function renoise.Application:show_message(message) end

---Shows an error dialog to the user.
---@param message string
function renoise.Application:show_error(message) end

---Shows a warning dialog to the user.
---@param message string
function renoise.Application:show_warning(message) end

---Shows a message in Renoise's status bar to the user.
---@param message string
function renoise.Application:show_status(message) end

---The modifier keys will be provided as a string.  
---Possible keys are dependent on the platform
--- * Windows : "shift", "alt", "control", "winkey"
--- * Linux : "shift", "alt", "control", "meta"
--- * Mac : "shift", "option", "control", "command"
---If multiple modifiers are held down, the string will be formatted as  
---"<key> + <key>"
---Their order will correspond to the following precedence
---`shift + alt/option + control + winkey/meta/command`  
---If no modifier is pressed, this will be an empty string
---@alias ModifierStates string
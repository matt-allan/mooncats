---@meta

---The Renoise application.
---@class renoise.Application
---
---**READ-ONLY** Access to the application's full log filename and path. Will
---already be opened for writing, but you nevertheless should be able to read
---from it.
---@field log_filename string
renoise.Application = {}

---Shows an info message dialog to the user.
---@param message string
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
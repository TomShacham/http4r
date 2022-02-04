/*
    http4r is a web toolkit
    Copyright (C) 2021 Tom Shacham

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod client;
pub mod http_message;
pub mod server;
mod pool;
pub mod logging_handler;
pub mod handler;
pub mod redirect_to_https_handler;
pub mod headers;
pub mod uri;
pub mod query;
pub mod codex;



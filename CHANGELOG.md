# 0.1.2
Date: _03/31/2025_
Enhanced database configuration and error handling features.

## New
Added dynamic database configuration options.
- Added `set_database_name` macro for dynamically setting the database name
- Added support for database name configuration via environment variables

## Fixes
Improved safety handling in code.
- Added unsafe block to `set_database_name` macro for proper safety marking

## Updated
Updated project metadata for future compatibility.
- Updated Rust edition from 2021 to 2024
- Bumped version from 0.1.1 to 0.1.2

# 0.1.1
Date: _03/26/2025_
Major improvements to error handling and server-database integration.

## New
Added example implementations and enhanced error responses.
- Added examples demonstrating server creation and database integration
- Added `/error` endpoint to showcase error handling capabilities
- Added JSON-formatted error responses with status codes

## Fixes
Resolved route configuration issues.
- Fixed route configuration order in Actix extension
- Fixed middleware configuration behavior

## Updated
Refined error handling and response formatting.
- Enhanced error handling with stacktrace support in debug mode
- Improved backtrace parsing to include absolute file paths
- Refactored error responses for better message clarity
- Simplified dependency version constraints

# 0.1.0
Date: _03/03/2025_
Initial release of the database-common-lib.

## New
Established core library infrastructure.
- Added Actix-web server with static file embedding support
- Added database connection module with config retrieval and pooling
- Added custom error handling with ResponseError support
- Added HTTP server creation functionality
- Added route configuration for Actix web app setup
- Added GNU GPLv3 license
- Added comprehensive README with usage examples

## Fixes
Initial release - no fixes yet.

## Updated
Improved initial implementation before first release.
- Refactored HTTP server creation for better flexibility and clarity
- Simplified Actix configuration by removing redundant code
- Optimized static directory handling via Data injection
- Implemented version constraints in dependencies
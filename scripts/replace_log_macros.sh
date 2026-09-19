#!/bin/bash
# Replace log::debug! with dure_debug!
find mobile/src -name "*.rs" -type f -exec sed -i 's/log::debug!/dure_debug!/g' {} +

# Replace log::info! with dure_info!
find mobile/src -name "*.rs" -type f -exec sed -i 's/log::info!/dure_info!/g' {} +

# Replace log::warn! with dure_warn!
find mobile/src -name "*.rs" -type f -exec sed -i 's/log::warn!/dure_warn!/g' {} +

# Replace log::error! with dure_error!
find mobile/src -name "*.rs" -type f -exec sed -i 's/log::error!/dure_error!/g' {} +

echo "Replacement complete. Run 'cargo check' to verify."

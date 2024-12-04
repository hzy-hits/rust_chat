#!/bin/bash

# Configuration variables
CONTAINER_NAME="rust_chat_db"    # Docker container name
DB_NAME="mydatabase"             # Database name
DB_USER="postgres"               # Database user
DB_PASSWORD="postgres"         # Database password (if needed)
HOST_PORT="5432"                 # Host PostgreSQL port
DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@localhost:$HOST_PORT/$DB_NAME"

# Check if container is running
echo "Checking container status..."
if ! docker ps --format "{{.Names}}" | grep -q "$CONTAINER_NAME"; then
    echo "Container $CONTAINER_NAME is not running. Please start the container and try again."
    exit 1
fi

# Force terminate all connections to the database
echo "Terminating all connections to database $DB_NAME..."
docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d postgres -c "
SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '$DB_NAME'
AND pid <> pg_backend_pid();
"

# Delete database
echo "Deleting database $DB_NAME..."
docker exec -i "$CONTAINER_NAME" dropdb -U "$DB_USER" "$DB_NAME" || echo "Database does not exist, skipping deletion step."

# Create database
echo "Creating database $DB_NAME..."
docker exec -i "$CONTAINER_NAME" createdb -U "$DB_USER" "$DB_NAME"

# Run migrations
echo "Running migrations..."
if DATABASE_URL=$DATABASE_URL sqlx migrate run; then
    echo "Migration successful!"
else
    echo "Migration failed, please check error messages."
    exit 1
fi

echo "Operation complete. Database $DB_NAME has been reset and all migrations applied."

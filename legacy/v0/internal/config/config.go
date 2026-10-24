package config

type Config struct {
	AppPort     string `envconfig:"APP_PORT" default:"8080"`
	AppEnv      string `envconfig:"APP_ENV" default:"development"`
	AppDebug    bool   `envconfig:"APP_DEBUG" default:"true"`
	DBHost      string `envconfig:"POSTGRES_HOST" default:"localhost"`
	DBPort      string `envconfig:"POSTGRES_PORT" default:"5432"`
	DBName      string `envconfig:"POSTGRES_DB" default:"equalizer_stake"`
	DBUser      string `envconfig:"POSTGRES_USER" default:"eqs"`
	DBPassword  string `envconfig:"POSTGRES_PASSWORD" default:"secret"`
	RedisHost   string `envconfig:"REDIS_HOST" default:"localhost"`
	RedisPort   string `envconfig:"REDIS_PORT" default:"6379"`
	JWTSecret   string `envconfig:"JWT_SECRET" default:"change-me-in-production"`
}

# Application
app-title = Astra Weather

# Tabs
tab-weather = Weather
tab-forecast = Forecast
tab-stations = Stations
tab-alerts = Alerts
tab-settings = Settings

# Weather View
weather-loading = Loading weather data...
weather-humidity = Humidity: {$value}%
weather-pressure = Pressure: {$value} hPa
weather-wind = Wind: {$value} m/s
weather-high = H: {$temp}
weather-low = L: {$temp}

# Forecast View
forecast-hourly-title = 7-Day Hourly Forecast
forecast-daily-title = 10-Day Daily Forecast
forecast-toggle-hourly = Hourly
forecast-toggle-daily = Daily
forecast-switch-to-daily = Switch to Daily Forecast
forecast-switch-to-hourly = Switch to Hourly Forecast
forecast-loading = Loading forecast data...
forecast-loading-message = Loading forecast...

# Detailed Sensor Data
sensor-data-title = Detailed Sensor Data
sensor-battery = Battery: {$voltage}V - {$status}
sensor-battery-excellent = Excellent
sensor-battery-good = Good
sensor-battery-fair = Fair
sensor-battery-low = Low
sensor-battery-critical = Critical
sensor-sea-level-pressure = Sea Level Pressure: {$value} {$unit}
sensor-lightning = Lightning: {$time}{$distance}{$count}
sensor-lightning-none = Lightning: No recent strikes
sensor-lightning-distance = at {$value} {$unit}
sensor-lightning-count = ({$count} strikes in 3h)
sensor-lightning-time-seconds = {$value}s ago
sensor-lightning-time-minutes = {$value}m ago
sensor-lightning-time-hours = {$value}h ago
sensor-lightning-time-days = {$value}d ago
sensor-wind = Wind: {$speed} {$unit} (gusting to {$gust} {$unit})
sensor-rain-detected = Rain: {$value} {$unit} in last hour
sensor-rain-none = No rain detected
sensor-uv-index = UV Index: {$value} ({$level})
sensor-uv-low = Low
sensor-uv-moderate = Moderate
sensor-uv-high = High
sensor-uv-very-high = Very High
sensor-uv-extreme = Extreme
sensor-solar-radiation = Solar Radiation: {$value} W/m²
sensor-brightness = Brightness: {$value} lux

# Alerts View
alerts-title = Weather Alerts & Notifications
alerts-notifications-label = All Notifications:
alerts-notifications-subtitle = (Lightning, Rain, High Wind)
alerts-notifications-on = 🔔 ON
alerts-notifications-off = 🔕 OFF

alerts-lightning-title = ⚡ Lightning Alerts
alerts-lightning-type = Type: {$strike_type}
alerts-lightning-strikes = Strikes: {$count}
alerts-lightning-distance = Distance: {$value} km
alerts-lightning-distance-unknown = Distance: Unknown
alerts-lightning-intensity = Intensity: {$level}
alerts-lightning-time = Time: {$time}
alerts-lightning-none = No recent lightning events

alerts-rain-title = 🌧️ Rain Alerts
alerts-rain-detected = Rain detected at station
alerts-rain-time = Time: {$time}
alerts-rain-none = No recent rain detected

alerts-wind-title = 💨 Wind Alerts
alerts-wind-high = ⚠️ HIGH WIND ALERT ⚠️
alerts-wind-recent = Recent Wind Reading
alerts-wind-speed = Speed: {$value} {$unit}
alerts-wind-direction = Direction: {$degrees}°
alerts-wind-time = Time: {$time}
alerts-wind-none = No recent wind alerts

# Stations View
stations-selection-required = Select a default station
stations-selection-loading = Loading stations...
stations-selection-no-stations = No stations found.
stations-fetch-button = Fetch Stations
stations-discover-button = Discover Stations
stations-select-button = Select
stations-header = Your Weather Stations
stations-title = Tempest Weather Stations
stations-subtitle = Manage and monitor your Tempest weather station network
stations-fetching = Fetching stations...
stations-none = No stations available. Click "Fetch My Stations" to load your stations.
stations-no-stations-available = No Stations Available
stations-no-stations-configured = No weather stations are currently configured.
stations-network-title = Station Network
stations-health = Health
stations-last-update = Last Update: {$time} ago
stations-signal = Signal: {$strength}%
stations-battery = Battery: {$level}%
stations-status-online = Online
stations-status-warning = Warning
stations-status-stale = Stale
stations-status-offline = Offline
stations-status-unknown = Unknown
stations-status-active = Active
stations-status-inactive = Inactive
stations-set-default = Set as Default
stations-current-default = Current Default
stations-active = Active
stations-location = Location: {$lat}, {$lon}
stations-location-label = Location
stations-serial-label = Serial Number
stations-last-updated-label = Last Updated
stations-last-updated-never = Never
stations-active-count = Active Stations
stations-total-count = Total Stations
stations-active-short = active
stations-total-short = total
stations-websocket-label = WebSocket
stations-websocket-connected = Connected
stations-websocket-waiting = Waiting for data...
stations-websocket-disconnected = Disconnected
stations-degraded-title = Station Connection Degraded
stations-degraded-explanation = Your weather station's hub or sensor is experiencing connectivity issues with WeatherFlow's servers. Weather data from the last successful update is still being displayed.
stations-degraded-first-time = Note: This is normal during initial setup. It may take a few minutes for your station to establish a stable connection.
stations-status-label = Status
stations-signal-label = Signal
stations-battery-label = Battery
stations-error-label = Error
stations-time-days = {$value} days ago
stations-time-hours = {$value} hours ago
stations-time-minutes = {$value} minutes ago
stations-time-just-now = Just now

# Settings View
settings-api-key-title = API Key Management
settings-api-key-new = Enter new API key:
settings-api-key-prompt = Enter your WeatherFlow Tempest API key:
settings-api-key-placeholder = Paste your API key here...
settings-api-key-save = Save
settings-api-key-cancel = Cancel
settings-api-key-how-to = How to get your API key:
settings-api-key-visit = 1. Visit
settings-api-key-login = 2. Log in and create a personal access token
settings-api-key-configured = ✅ API Key configured
settings-api-key-change = Change
settings-api-key-remove = Remove
settings-api-key-secure-message = Your API key is securely stored and the app is fully functional.
settings-api-key-not-configured = ❌ No API Key configured
settings-api-key-add-prompt = Please add your WeatherFlow Tempest API key to use this applet.
settings-api-key-add-button = Add API Key

settings-units-title = Units
settings-temperature = Temperature Unit
settings-wind-speed = Wind Speed Unit
settings-pressure = Pressure Unit
settings-precipitation = Precipitation Unit
settings-distance = Distance Unit

settings-station-title = Weather Station
settings-station-none = No stations available
settings-refresh-interval-title = Refresh Interval
settings-location-title = Location
settings-location-auto-on = Auto Location: ON
settings-location-auto-off = Auto Location: OFF
settings-location-current = Current: {$lat}, {$lon}
settings-location-none = No location set
settings-version = Version

# Units
unit-celsius = °C
unit-fahrenheit = °F
unit-mps = m/s
unit-kph = km/h
unit-mph = mph
unit-knots = knots
unit-mb = mb
unit-inhg = inHg
unit-mm = mm
unit-in = in
unit-km = km
unit-mi = mi

# Common
common-loading = Loading...
common-error = Error
common-refresh = Refresh
common-close = Close
common-ok = OK
common-cancel = Cancel

# About Tab
tab-about = About
about-tagline = Real-time weather from Tempest Weather Stations
about-version = Version
about-description = A COSMIC desktop panel applet for displaying real-time weather data from Tempest Weather Stations with WebSocket integration and comprehensive internationalization support.
about-repository = Repository
about-license = License

# Lockdown View
lockdown-title = API Key Required
lockdown-message = Please configure your Tempest API key to use this applet.
lockdown-configure = Configure API Key

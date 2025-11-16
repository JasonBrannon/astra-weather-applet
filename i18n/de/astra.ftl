# Application
app-title = Astra Wetter

# Tabs
tab-weather = Wetter
tab-forecast = Vorhersage
tab-stations = Stationen
tab-alerts = Warnungen
tab-settings = Einstellungen

# Weather View
weather-loading = Wetterdaten werden geladen...
weather-humidity = Luftfeuchtigkeit: {$value}%
weather-pressure = Luftdruck: {$value} hPa
weather-wind = Wind: {$value} m/s
weather-high = H: {$temp}
weather-low = T: {$temp}

# Forecast View
forecast-hourly-title = 7-Tage-Stundenvorhersage
forecast-daily-title = 10-Tage-Tagesvorhersage
forecast-toggle-hourly = Stündlich
forecast-toggle-daily = Täglich
forecast-switch-to-daily = Zur Tagesvorhersage wechseln
forecast-switch-to-hourly = Zur Stundenvorhersage wechseln
forecast-loading = Vorhersagedaten werden geladen...
forecast-loading-message = Vorhersage wird geladen...

# Detailed Sensor Data
sensor-data-title = Detaillierte Sensordaten
sensor-battery = Batterie: {$voltage}V - {$status}
sensor-battery-excellent = Ausgezeichnet
sensor-battery-good = Gut
sensor-battery-fair = Befriedigend
sensor-battery-low = Niedrig
sensor-battery-critical = Kritisch
sensor-sea-level-pressure = Luftdruck auf Meereshöhe: {$value} {$unit}
sensor-lightning = Blitz: {$time}{$distance}{$count}
sensor-lightning-none = Blitz: Keine aktuellen Einschläge
sensor-lightning-distance = in {$value} {$unit}
sensor-lightning-count = ({$count} Einschläge in 3h)
sensor-lightning-time-seconds = vor {$value}s
sensor-lightning-time-minutes = vor {$value}m
sensor-lightning-time-hours = vor {$value}h
sensor-lightning-time-days = vor {$value}d
sensor-wind = Wind: {$speed} {$unit} (Böen bis {$gust} {$unit})
sensor-rain-detected = Regen: {$value} {$unit} in der letzten Stunde
sensor-rain-none = Kein Regen erkannt
sensor-uv-index = UV-Index: {$value} ({$level})
sensor-uv-low = Niedrig
sensor-uv-moderate = Mäßig
sensor-uv-high = Hoch
sensor-uv-very-high = Sehr hoch
sensor-uv-extreme = Extrem
sensor-solar-radiation = Sonneneinstrahlung: {$value} W/m²
sensor-brightness = Helligkeit: {$value} lux

# Alerts View
alerts-title = Wetterwarnungen & Benachrichtigungen
alerts-notifications-label = Alle Benachrichtigungen:
alerts-notifications-subtitle = (Blitz, Regen, Starker Wind)
alerts-notifications-on = 🔔 EIN
alerts-notifications-off = 🔕 AUS

alerts-lightning-title = ⚡ Blitzwarnungen
alerts-lightning-type = Typ: {$strike_type}
alerts-lightning-strikes = Einschläge: {$count}
alerts-lightning-distance = Entfernung: {$value} km
alerts-lightning-distance-unknown = Entfernung: Unbekannt
alerts-lightning-intensity = Intensität: {$level}
alerts-lightning-time = Zeit: {$time}
alerts-lightning-none = Keine aktuellen Blitzereignisse

alerts-rain-title = 🌧️ Regenwarnungen
alerts-rain-detected = Regen an der Station erkannt
alerts-rain-time = Zeit: {$time}
alerts-rain-none = Kein aktueller Regen erkannt

alerts-wind-title = 💨 Windwarnungen
alerts-wind-high = ⚠️ STARKWINDWARNUNG ⚠️
alerts-wind-recent = Aktuelle Windmessung
alerts-wind-speed = Geschwindigkeit: {$value} {$unit}
alerts-wind-direction = Richtung: {$degrees}°
alerts-wind-time = Zeit: {$time}
alerts-wind-none = Keine aktuellen Windwarnungen

# Stations View
stations-selection-required = Standardstation wählen
stations-selection-loading = Stationen werden geladen...
stations-selection-no-stations = Keine Stationen gefunden.
stations-fetch-button = Stationen abrufen
stations-discover-button = Stationen entdecken
stations-select-button = Auswählen
stations-header = Ihre Wetterstationen
stations-title = Tempest Wetterstationen
stations-subtitle = Verwalten und überwachen Sie Ihr Tempest-Wetterstationsnetzwerk
stations-fetching = Stationen werden abgerufen...
stations-none = Keine Stationen verfügbar. Klicken Sie auf "Meine Stationen abrufen", um Ihre Stationen zu laden.
stations-no-stations-available = Keine Stationen verfügbar
stations-no-stations-configured = Derzeit sind keine Wetterstationen konfiguriert.
stations-network-title = Stationsnetzwerk
stations-health = Zustand
stations-last-update = Letzte Aktualisierung: vor {$time}
stations-signal = Signal: {$strength}%
stations-battery = Batterie: {$level}%
stations-status-online = Online
stations-status-warning = Warnung
stations-status-stale = Veraltet
stations-status-offline = Offline
stations-status-unknown = Unbekannt
stations-status-active = Aktiv
stations-status-inactive = Inaktiv
stations-set-default = Als Standard festlegen
stations-current-default = Aktuelle Standardstation
stations-active = Aktiv
stations-location = Standort: {$lat}, {$lon}
stations-location-label = Standort
stations-serial-label = Seriennummer
stations-last-updated-label = Zuletzt aktualisiert
stations-last-updated-never = Niemals
stations-active-count = Aktive Stationen
stations-total-count = Stationen gesamt
stations-active-short = aktiv
stations-total-short = gesamt
stations-websocket-label = WebSocket
stations-websocket-connected = Verbunden
stations-websocket-waiting = Warte auf Daten...
stations-websocket-disconnected = Getrennt
stations-degraded-title = Stationsverbindung eingeschränkt
stations-degraded-explanation = Der Hub oder Sensor Ihrer Wetterstation hat Verbindungsprobleme mit den WeatherFlow-Servern. Wetterdaten der letzten erfolgreichen Aktualisierung werden weiterhin angezeigt.
stations-degraded-first-time = Hinweis: Dies ist bei der ersten Einrichtung normal. Es kann einige Minuten dauern, bis Ihre Station eine stabile Verbindung aufbaut.
stations-status-label = Status
stations-signal-label = Signal
stations-battery-label = Batterie
stations-error-label = Fehler
stations-time-days = vor {$value} Tagen
stations-time-hours = vor {$value} Stunden
stations-time-minutes = vor {$value} Minuten
stations-time-just-now = Gerade eben

# Settings View
settings-api-key-title = API-Schlüsselverwaltung
settings-api-key-new = Neuen API-Schlüssel eingeben:
settings-api-key-prompt = Geben Sie Ihren WeatherFlow Tempest API-Schlüssel ein:
settings-api-key-placeholder = API-Schlüssel hier einfügen...
settings-api-key-save = Speichern
settings-api-key-cancel = Abbrechen
settings-api-key-how-to = So erhalten Sie Ihren API-Schlüssel:
settings-api-key-visit = 1. Besuchen Sie
settings-api-key-login = 2. Melden Sie sich an und erstellen Sie ein persönliches Zugriffstoken
settings-api-key-configured = ✅ API-Schlüssel konfiguriert
settings-api-key-change = Ändern
settings-api-key-remove = Entfernen
settings-api-key-secure-message = Ihr API-Schlüssel ist sicher gespeichert und die App ist voll funktionsfähig.
settings-api-key-not-configured = ❌ Kein API-Schlüssel konfiguriert
settings-api-key-add-prompt = Bitte fügen Sie Ihren WeatherFlow Tempest API-Schlüssel hinzu, um dieses Applet zu verwenden.
settings-api-key-add-button = API-Schlüssel hinzufügen

settings-units-title = Einheiten
settings-temperature = Temperatureinheit
settings-wind-speed = Windgeschwindigkeitseinheit
settings-pressure = Luftdruckeinheit
settings-precipitation = Niederschlagseinheit
settings-distance = Entfernungseinheit

settings-station-title = Wetterstation
settings-station-none = Keine Stationen verfügbar
settings-refresh-interval-title = Aktualisierungsintervall
settings-location-title = Standort
settings-location-auto-on = Automatischer Standort: EIN
settings-location-auto-off = Automatischer Standort: AUS
settings-location-current = Aktuell: {$lat}, {$lon}
settings-location-none = Kein Standort festgelegt
settings-version = Version

# Units
unit-celsius = °C
unit-fahrenheit = °F
unit-mps = m/s
unit-kph = km/h
unit-mph = mph
unit-knots = Knoten
unit-mb = mb
unit-inhg = inHg
unit-mm = mm
unit-in = in
unit-km = km
unit-mi = mi

# Common
common-loading = Lädt...
common-error = Fehler
common-refresh = Aktualisieren
common-close = Schließen
common-ok = OK
common-cancel = Abbrechen

# About Tab
tab-about = Über
about-tagline = Echtzeitwetter von Tempest-Wetterstationen
about-version = Version
about-description = Ein COSMIC-Desktop-Panel-Applet zur Anzeige von Echtzeitwetterdaten von Tempest-Wetterstationen mit WebSocket-Integration und umfassender Internationalisierungsunterstützung.
about-repository = Repository
about-license = Lizenz

# Lockdown View
lockdown-title = API-Schlüssel erforderlich
lockdown-message = Bitte konfigurieren Sie Ihren Tempest API-Schlüssel, um dieses Applet zu verwenden.
lockdown-configure = API-Schlüssel konfigurieren

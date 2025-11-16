# Application
app-title = Astra Meteorológica

# Tabs
tab-weather = Clima
tab-forecast = Pronóstico
tab-stations = Estaciones
tab-alerts = Alertas
tab-settings = Configuración

# Weather View
weather-loading = Cargando datos meteorológicos...
weather-humidity = Humedad: {$value}%
weather-pressure = Presión: {$value} hPa
weather-wind = Viento: {$value} m/s
weather-high = Máx: {$temp}
weather-low = Mín: {$temp}

# Forecast View
forecast-hourly-title = Pronóstico Horario de 7 Días
forecast-daily-title = Pronóstico Diario de 10 Días
forecast-toggle-hourly = Por Hora
forecast-toggle-daily = Diario
forecast-switch-to-daily = Cambiar a Pronóstico Diario
forecast-switch-to-hourly = Cambiar a Pronóstico Horario
forecast-loading = Cargando datos de pronóstico...
forecast-loading-message = Cargando pronóstico...

# Detailed Sensor Data
sensor-data-title = Datos Detallados del Sensor
sensor-battery = Batería: {$voltage}V - {$status}
sensor-battery-excellent = Excelente
sensor-battery-good = Buena
sensor-battery-fair = Aceptable
sensor-battery-low = Baja
sensor-battery-critical = Crítica
sensor-sea-level-pressure = Presión al Nivel del Mar: {$value} {$unit}
sensor-lightning = Relámpagos: {$time}{$distance}{$count}
sensor-lightning-none = Relámpagos: Sin detecciones recientes
sensor-lightning-distance = a {$value} {$unit}
sensor-lightning-count = ({$count} rayos en 3h)
sensor-lightning-time-seconds = hace {$value}s
sensor-lightning-time-minutes = hace {$value}m
sensor-lightning-time-hours = hace {$value}h
sensor-lightning-time-days = hace {$value}d
sensor-wind = Viento: {$speed} {$unit} (ráfagas hasta {$gust} {$unit})
sensor-rain-detected = Lluvia: {$value} {$unit} en la última hora
sensor-rain-none = Sin lluvia detectada
sensor-uv-index = Índice UV: {$value} ({$level})
sensor-uv-low = Bajo
sensor-uv-moderate = Moderado
sensor-uv-high = Alto
sensor-uv-very-high = Muy Alto
sensor-uv-extreme = Extremo
sensor-solar-radiation = Radiación Solar: {$value} W/m²
sensor-brightness = Brillo: {$value} lux

# Alerts View
alerts-title = Alertas y Notificaciones Meteorológicas
alerts-notifications-label = Todas las Notificaciones:
alerts-notifications-subtitle = (Relámpagos, Lluvia, Viento Fuerte)
alerts-notifications-on = 🔔 ACTIVADO
alerts-notifications-off = 🔕 DESACTIVADO

alerts-lightning-title = ⚡ Alertas de Relámpagos
alerts-lightning-type = Tipo: {$strike_type}
alerts-lightning-strikes = Rayos: {$count}
alerts-lightning-distance = Distancia: {$value} km
alerts-lightning-distance-unknown = Distancia: Desconocida
alerts-lightning-intensity = Intensidad: {$level}
alerts-lightning-time = Hora: {$time}
alerts-lightning-none = Sin eventos de relámpagos recientes

alerts-rain-title = 🌧️ Alertas de Lluvia
alerts-rain-detected = Lluvia detectada en la estación
alerts-rain-time = Hora: {$time}
alerts-rain-none = Sin lluvia reciente detectada

alerts-wind-title = 💨 Alertas de Viento
alerts-wind-high = ⚠️ ALERTA DE VIENTO FUERTE ⚠️
alerts-wind-recent = Lectura de Viento Reciente
alerts-wind-speed = Velocidad: {$value} {$unit}
alerts-wind-direction = Dirección: {$degrees}°
alerts-wind-time = Hora: {$time}
alerts-wind-none = Sin alertas de viento recientes

# Stations View
stations-selection-required = Seleccionar una estación predeterminada
stations-selection-loading = Cargando estaciones...
stations-selection-no-stations = No se encontraron estaciones.
stations-fetch-button = Obtener Estaciones
stations-discover-button = Descubrir Estaciones
stations-select-button = Seleccionar
stations-header = Sus Estaciones Meteorológicas
stations-title = Estaciones Meteorológicas Tempest
stations-subtitle = Administre y monitoree su red de estaciones meteorológicas Tempest
stations-fetching = Obteniendo estaciones...
stations-none = No hay estaciones disponibles. Haga clic en "Obtener Mis Estaciones" para cargar sus estaciones.
stations-no-stations-available = No Hay Estaciones Disponibles
stations-no-stations-configured = Actualmente no hay estaciones meteorológicas configuradas.
stations-network-title = Red de Estaciones
stations-health = Salud
stations-last-update = Última Actualización: hace {$time}
stations-signal = Señal: {$strength}%
stations-battery = Batería: {$level}%
stations-status-online = En Línea
stations-status-warning = Advertencia
stations-status-stale = Obsoleta
stations-status-offline = Fuera de Línea
stations-status-unknown = Desconocido
stations-status-active = Activa
stations-status-inactive = Inactiva
stations-set-default = Establecer como Predeterminada
stations-current-default = Predeterminada Actual
stations-active = Activa
stations-location = Ubicación: {$lat}, {$lon}
stations-location-label = Ubicación
stations-serial-label = Número de Serie
stations-last-updated-label = Última Actualización
stations-last-updated-never = Nunca
stations-active-count = Estaciones Activas
stations-total-count = Total de Estaciones
stations-active-short = activas
stations-total-short = total
stations-websocket-label = WebSocket
stations-websocket-connected = Conectado
stations-websocket-waiting = Esperando datos...
stations-websocket-disconnected = Desconectado
stations-degraded-title = Conexión de Estación Degradada
stations-degraded-explanation = El hub o sensor de su estación meteorológica está experimentando problemas de conectividad con los servidores de WeatherFlow. Aún se muestran los datos meteorológicos de la última actualización exitosa.
stations-degraded-first-time = Nota: Esto es normal durante la configuración inicial. Puede tomar unos minutos para que su estación establezca una conexión estable.
stations-status-label = Estado
stations-signal-label = Señal
stations-battery-label = Batería
stations-error-label = Error
stations-time-days = hace {$value} días
stations-time-hours = hace {$value} horas
stations-time-minutes = hace {$value} minutos
stations-time-just-now = Justo ahora

# Settings View
settings-api-key-title = Administración de Clave API
settings-api-key-new = Ingrese nueva clave API:
settings-api-key-prompt = Ingrese su clave API de WeatherFlow Tempest:
settings-api-key-placeholder = Pegue su clave API aquí...
settings-api-key-save = Guardar
settings-api-key-cancel = Cancelar
settings-api-key-how-to = Cómo obtener su clave API:
settings-api-key-visit = 1. Visite
settings-api-key-login = 2. Inicie sesión y cree un token de acceso personal
settings-api-key-configured = ✅ Clave API configurada
settings-api-key-change = Cambiar
settings-api-key-remove = Eliminar
settings-api-key-secure-message = Su clave API está almacenada de forma segura y la aplicación está completamente funcional.
settings-api-key-not-configured = ❌ No hay clave API configurada
settings-api-key-add-prompt = Por favor agregue su clave API de WeatherFlow Tempest para usar esta aplicación.
settings-api-key-add-button = Agregar Clave API

settings-units-title = Unidades
settings-temperature = Unidad de Temperatura
settings-wind-speed = Unidad de Velocidad del Viento
settings-pressure = Unidad de Presión
settings-precipitation = Unidad de Precipitación
settings-distance = Unidad de Distancia

settings-station-title = Estación Meteorológica
settings-station-none = No hay estaciones disponibles
settings-refresh-interval-title = Intervalo de Actualización
settings-location-title = Ubicación
settings-location-auto-on = Ubicación Automática: ACTIVADA
settings-location-auto-off = Ubicación Automática: DESACTIVADA
settings-location-current = Actual: {$lat}, {$lon}
settings-location-none = No hay ubicación establecida
settings-version = Versión

# Units
unit-celsius = °C
unit-fahrenheit = °F
unit-mps = m/s
unit-kph = km/h
unit-mph = mph
unit-knots = nudos
unit-mb = mb
unit-inhg = inHg
unit-mm = mm
unit-in = in
unit-km = km
unit-mi = mi

# Common
common-loading = Cargando...
common-error = Error
common-refresh = Actualizar
common-close = Cerrar
common-ok = Aceptar
common-cancel = Cancelar

# About Tab
tab-about = Acerca de
about-tagline = Clima en tiempo real de Estaciones Meteorológicas Tempest
about-version = Versión
about-description = Un applet de panel de escritorio COSMIC para mostrar datos meteorológicos en tiempo real de Estaciones Meteorológicas Tempest con integración WebSocket y soporte completo de internacionalización.
about-repository = Repositorio
about-license = Licencia

# Lockdown View
lockdown-title = Clave API Requerida
lockdown-message = Por favor configure su clave API de Tempest para usar esta aplicación.
lockdown-configure = Configurar Clave API

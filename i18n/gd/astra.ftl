# Application
app-title = Astra Sìde

# Tabs
tab-weather = Sìde
tab-forecast = Ro-aithris
tab-stations = Stèiseanan
tab-alerts = Rabhaidhean
tab-settings = Roghainnean

# Weather View
weather-loading = A' luchdadh dàta na sìde...
weather-humidity = Taiseachd: {$value}%
weather-pressure = Cuideam: {$value} hPa
weather-wind = Gaoth: {$value} m/s
weather-high = H: {$temp}
weather-low = L: {$temp}

# Forecast View
forecast-hourly-title = Ro-aithris Uaireil 7 Làithean
forecast-daily-title = Ro-aithris Làitheil 10 Làithean
forecast-toggle-hourly = Uaireil
forecast-toggle-daily = Làitheil
forecast-switch-to-daily = Atharraich gu Ro-aithris Làitheil
forecast-switch-to-hourly = Atharraich gu Ro-aithris Uaireil
forecast-loading = A' luchdadh dàta na ro-aithris...
forecast-loading-message = A' luchdadh na ro-aithris...

# Detailed Sensor Data
sensor-data-title = Dàta Mionaideach an t-Seannsair
sensor-battery = Bataraidh: {$voltage}V - {$status}
sensor-battery-excellent = Sàr-mhath
sensor-battery-good = Math
sensor-battery-fair = Meadhanach
sensor-battery-low = Ìosal
sensor-battery-critical = Èiginneach
sensor-sea-level-pressure = Cuideam Aig Ìre na Mara: {$value} {$unit}
sensor-lightning = Dealanach: {$time}{$distance}{$count}
sensor-lightning-none = Dealanach: Gun bhuillean o chionn ghoirid
sensor-lightning-distance = aig {$value} {$unit}
sensor-lightning-count = ({$count} buillean ann an 3u)
sensor-lightning-time-seconds = {$value}d air ais
sensor-lightning-time-minutes = {$value}m air ais
sensor-lightning-time-hours = {$value}u air ais
sensor-lightning-time-days = {$value}l air ais
sensor-wind = Gaoth: {$speed} {$unit} (a' sèideadh gu {$gust} {$unit})
sensor-rain-detected = Uisge: {$value} {$unit} anns an uair a dh'fhalbh
sensor-rain-none = Gun uisge air a lorg
sensor-uv-index = Clàr-innse UV: {$value} ({$level})
sensor-uv-low = Ìosal
sensor-uv-moderate = Meadhanach
sensor-uv-high = Àrd
sensor-uv-very-high = Glè Àrd
sensor-uv-extreme = Anabarrach
sensor-solar-radiation = Rèididheachd na Grèine: {$value} W/m²
sensor-brightness = Soilleireachd: {$value} lux

# Alerts View
alerts-title = Rabhaidhean Sìde & Brathan
alerts-notifications-label = A h-Uile Brath:
alerts-notifications-subtitle = (Dealanach, Uisge, Gaoth Làidir)
alerts-notifications-on = 🔔 AIR
alerts-notifications-off = 🔕 DHETH

alerts-lightning-title = ⚡ Rabhaidhean Dealanaich
alerts-lightning-type = Seòrsa: {$strike_type}
alerts-lightning-strikes = Buillean: {$count}
alerts-lightning-distance = Astar: {$value} km
alerts-lightning-distance-unknown = Astar: Neo-aithnichte
alerts-lightning-intensity = Dèineas: {$level}
alerts-lightning-time = Àm: {$time}
alerts-lightning-none = Gun tachartasan dealanaich o chionn ghoirid

alerts-rain-title = 🌧️ Rabhaidhean Uisge
alerts-rain-detected = Uisge air a lorg aig an stèisean
alerts-rain-time = Àm: {$time}
alerts-rain-none = Gun uisge air a lorg o chionn ghoirid

alerts-wind-title = 💨 Rabhaidhean Gaoithe
alerts-wind-high = ⚠️ RABHADH GAOTH LÀIDIR ⚠️
alerts-wind-recent = Leughadh Gaoithe O Chionn Ghoirid
alerts-wind-speed = Astar: {$value} {$unit}
alerts-wind-direction = Comhair: {$degrees}°
alerts-wind-time = Àm: {$time}
alerts-wind-none = Gun rabhaidhean gaoithe o chionn ghoirid

# Stations View
stations-selection-required = Tagh stèisean bunaiteach
stations-selection-loading = A' luchdadh stèiseanan...
stations-selection-no-stations = Cha deach stèiseanan a lorg.
stations-fetch-button = Faigh Stèiseanan
stations-discover-button = Lorg Stèiseanan
stations-select-button = Tagh
stations-header = Na Stèiseanan Sìde Agad
stations-title = Stèiseanan Sìde Tempest
stations-subtitle = Rianaich agus sùil air lìonra stèiseanan sìde Tempest agad
stations-fetching = A' faighinn stèiseanan...
stations-none = Chan eil stèiseanan ri làimh. Brùth air "Faigh Na Stèiseanan Agam" gus na stèiseanan agad a luchdadh.
stations-no-stations-available = Chan Eil Stèiseanan Ri Làimh
stations-no-stations-configured = Chan eil stèiseanan sìde air an rèiteachadh an-dràsta.
stations-network-title = Lìonra Stèiseanan
stations-health = Slàinte
stations-last-update = An Ùrachadh Mu Dheireadh: {$time} air ais
stations-signal = Comharra: {$strength}%
stations-battery = Bataraidh: {$level}%
stations-status-online = Air-loidhne
stations-status-warning = Rabhadh
stations-status-stale = Sean
stations-status-offline = Far-loidhne
stations-status-unknown = Neo-aithnichte
stations-status-active = Gnìomhach
stations-status-inactive = Neo-ghnìomhach
stations-set-default = Suidhich mar Bhunasach
stations-current-default = Bunasach an-dràsta
stations-active = Gnìomhach
stations-location = Àite: {$lat}, {$lon}
stations-location-label = Àite
stations-serial-label = Àireamh Shreathach
stations-last-updated-label = Ùrachadh Mu Dheireadh
stations-last-updated-never = A-riamh
stations-active-count = Stèiseanan Gnìomhach
stations-total-count = Stèiseanan Uile Gu Lèir
stations-active-short = gnìomhach
stations-total-short = iomlan
stations-websocket-label = WebSocket
stations-websocket-connected = Ceangailte
stations-websocket-waiting = A' feitheamh ri dàta...
stations-websocket-disconnected = Neo-cheangailte
stations-degraded-title = Ceangal Stèisein Air A Lagachadh
stations-degraded-explanation = Tha duilgheadasan ceangail aig hub no sensor an stèisein sìde agad le frithealaichean WeatherFlow. Tha dàta na sìde bhon ùrachadh shoirbheachail mu dheireadh fhathast ga shealltainn.
stations-degraded-first-time = Nota: Tha seo nàdarra rè a' chiad rèiteachaidh. Dh'fhaodadh e mionaidean a ghabhail gus an stèid an stèisean agad ceangal seasmhach.
stations-status-label = Staid
stations-signal-label = Comharra
stations-battery-label = Bataraidh
stations-error-label = Mearachd
stations-time-days = {$value} làithean air ais
stations-time-hours = {$value} uairean air ais
stations-time-minutes = {$value} mionaidean air ais
stations-time-just-now = An-dràsta fhèin

# Settings View
settings-api-key-title = Riaghladh Iuchair API
settings-api-key-new = Cuir a-steach iuchair API ùr:
settings-api-key-prompt = Cuir a-steach an iuchair API WeatherFlow Tempest agad:
settings-api-key-placeholder = Cuir an iuchair API agad an seo...
settings-api-key-save = Sàbhail
settings-api-key-cancel = Sguir dheth
settings-api-key-how-to = Mar a gheibh thu an iuchair API agad:
settings-api-key-visit = 1. Tadhail air
settings-api-key-login = 2. Clàraich a-steach agus cruthaich tòcan ruigsinne pearsanta
settings-api-key-configured = ✅ Iuchair API air a rèiteachadh
settings-api-key-change = Atharraich
settings-api-key-remove = Thoir air falbh
settings-api-key-secure-message = Tha an iuchair API agad air a stòradh gu tèarainte agus tha an app ag obair gu h-iomlan.
settings-api-key-not-configured = ❌ Gun iuchair API air a rèiteachadh
settings-api-key-add-prompt = Feuch an cuir thu an iuchair API WeatherFlow Tempest agad gus an applet seo a chleachdadh.
settings-api-key-add-button = Cuir Iuchair API Ris

settings-units-title = Aonadan
settings-temperature = Aonad Teòthachd
settings-wind-speed = Aonad Astar na Gaoithe
settings-pressure = Aonad Cuideam
settings-precipitation = Aonad Sileadh
settings-distance = Aonad Astar

settings-station-title = Stèisean Sìde
settings-station-none = Chan eil stèiseanan ri làimh
settings-refresh-interval-title = Eadaramh Ùrachaidh
settings-location-title = Àite
settings-location-auto-on = Àite Fèin-obrachail: AIR
settings-location-auto-off = Àite Fèin-obrachail: DHETH
settings-location-current = An-dràsta: {$lat}, {$lon}
settings-location-none = Gun àite air a shuidheachadh
settings-version = Tionndadh

# Units
unit-celsius = °C
unit-fahrenheit = °F
unit-mps = m/s
unit-kph = km/u
unit-mph = msu
unit-knots = knots
unit-mb = mb
unit-inhg = inHg
unit-mm = mm
unit-in = in
unit-km = km
unit-mi = mi

# Common
common-loading = A' luchdadh...
common-error = Mearachd
common-refresh = Ùraich
common-close = Dùin
common-ok = Ceart ma-thà
common-cancel = Sguir dheth

# About Tab
tab-about = Mu dheidhinn
about-tagline = Sìde fìor-ùine bho Stèiseanan Sìde Tempest
about-version = Tionndadh
about-description = Applet panail deasg COSMIC airson dàta sìde fìor-ùine a thaisbeanadh bho Stèiseanan Sìde Tempest le amalachadh WebSocket agus taic eadar-nàiseantachaidh coileanta.
about-repository = Stòr-dàta
about-license = Ceadachd

# Lockdown View
lockdown-title = Feumaidh Iuchair API
lockdown-message = Feuch an rèitich thu an iuchair API Tempest agad gus an applet seo a chleachdadh.
lockdown-configure = Rèitich Iuchair API

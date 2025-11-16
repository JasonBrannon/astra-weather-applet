# Application
app-title = Astra天気

# Tabs
tab-weather = 天気
tab-forecast = 予報
tab-stations = 観測所
tab-alerts = 警報
tab-settings = 設定

# Weather View
weather-loading = 気象データを読み込んでいます...
weather-humidity = 湿度: {$value}%
weather-pressure = 気圧: {$value} hPa
weather-wind = 風速: {$value} m/s
weather-high = 最高: {$temp}
weather-low = 最低: {$temp}

# Forecast View
forecast-hourly-title = 7日間の時間別予報
forecast-daily-title = 10日間の日別予報
forecast-toggle-hourly = 時間別
forecast-toggle-daily = 日別
forecast-switch-to-daily = 日別予報に切り替え
forecast-switch-to-hourly = 時間別予報に切り替え
forecast-loading = 予報データを読み込んでいます...
forecast-loading-message = 予報を読み込んでいます...

# Detailed Sensor Data
sensor-data-title = 詳細センサーデータ
sensor-battery = バッテリー: {$voltage}V - {$status}
sensor-battery-excellent = 優良
sensor-battery-good = 良好
sensor-battery-fair = 普通
sensor-battery-low = 低下
sensor-battery-critical = 危機的
sensor-sea-level-pressure = 海面気圧: {$value} {$unit}
sensor-lightning = 落雷: {$time}{$distance}{$count}
sensor-lightning-none = 落雷: 最近の落雷なし
sensor-lightning-distance = {$value} {$unit}先
sensor-lightning-count = (3時間で{$count}回)
sensor-lightning-time-seconds = {$value}秒前
sensor-lightning-time-minutes = {$value}分前
sensor-lightning-time-hours = {$value}時間前
sensor-lightning-time-days = {$value}日前
sensor-wind = 風速: {$speed} {$unit} (突風 {$gust} {$unit})
sensor-rain-detected = 降雨: 過去1時間に{$value} {$unit}
sensor-rain-none = 降雨なし
sensor-uv-index = UV指数: {$value} ({$level})
sensor-uv-low = 低い
sensor-uv-moderate = 中程度
sensor-uv-high = 高い
sensor-uv-very-high = 非常に高い
sensor-uv-extreme = 極端に高い
sensor-solar-radiation = 日射量: {$value} W/m²
sensor-brightness = 照度: {$value} lux

# Alerts View
alerts-title = 気象警報・通知
alerts-notifications-label = すべての通知:
alerts-notifications-subtitle = (落雷、降雨、強風)
alerts-notifications-on = 🔔 オン
alerts-notifications-off = 🔕 オフ

alerts-lightning-title = ⚡ 落雷警報
alerts-lightning-type = 種類: {$strike_type}
alerts-lightning-strikes = 回数: {$count}
alerts-lightning-distance = 距離: {$value} km
alerts-lightning-distance-unknown = 距離: 不明
alerts-lightning-intensity = 強度: {$level}
alerts-lightning-time = 時刻: {$time}
alerts-lightning-none = 最近の落雷なし

alerts-rain-title = 🌧️ 降雨警報
alerts-rain-detected = 観測所で降雨を検出
alerts-rain-time = 時刻: {$time}
alerts-rain-none = 最近の降雨なし

alerts-wind-title = 💨 強風警報
alerts-wind-high = ⚠️ 強風警報 ⚠️
alerts-wind-recent = 最近の風速観測
alerts-wind-speed = 風速: {$value} {$unit}
alerts-wind-direction = 風向: {$degrees}°
alerts-wind-time = 時刻: {$time}
alerts-wind-none = 最近の強風警報なし

# Stations View
stations-selection-required = デフォルト観測所を選択
stations-selection-loading = 観測所を読み込み中...
stations-selection-no-stations = 観測所が見つかりません。
stations-fetch-button = 観測所を取得
stations-discover-button = 観測所を検出
stations-select-button = 選択
stations-header = あなたの気象観測所
stations-title = Tempest気象観測所
stations-subtitle = Tempest気象観測所ネットワークを管理および監視
stations-fetching = 観測所を取得中...
stations-none = 利用可能な観測所がありません。「観測所を取得」をクリックして観測所を読み込んでください。
stations-no-stations-available = 利用可能な観測所がありません
stations-no-stations-configured = 現在設定されている気象観測所がありません。
stations-network-title = 観測所ネットワーク
stations-health = 状態
stations-last-update = 最終更新: {$time}前
stations-signal = 信号: {$strength}%
stations-battery = バッテリー: {$level}%
stations-status-online = オンライン
stations-status-warning = 警告
stations-status-stale = 古い
stations-status-offline = オフライン
stations-status-unknown = 不明
stations-status-active = 稼働中
stations-status-inactive = 非稼働
stations-set-default = デフォルトに設定
stations-current-default = 現在のデフォルト
stations-active = 稼働中
stations-location = 場所: {$lat}, {$lon}
stations-location-label = 場所
stations-serial-label = シリアル番号
stations-last-updated-label = 最終更新
stations-last-updated-never = 未更新
stations-active-count = 稼働中の観測所
stations-total-count = 観測所の合計
stations-active-short = 稼働中
stations-total-short = 合計
stations-websocket-label = WebSocket
stations-websocket-connected = 接続済み
stations-websocket-waiting = データ待機中...
stations-websocket-disconnected = 切断
stations-degraded-title = 観測所の接続が劣化しています
stations-degraded-explanation = 気象観測所のハブまたはセンサーがWeatherFlowサーバーとの接続に問題があります。最後の正常な更新からの気象データが引き続き表示されます。
stations-degraded-first-time = 注意: 初期セットアップ中はこれは正常です。観測所が安定した接続を確立するまで数分かかる場合があります。
stations-status-label = ステータス
stations-signal-label = 信号
stations-battery-label = バッテリー
stations-error-label = エラー
stations-time-days = {$value}日前
stations-time-hours = {$value}時間前
stations-time-minutes = {$value}分前
stations-time-just-now = たった今

# Settings View
settings-api-key-title = APIキー管理
settings-api-key-new = 新しいAPIキーを入力:
settings-api-key-prompt = WeatherFlow Tempest APIキーを入力してください:
settings-api-key-placeholder = APIキーをここに貼り付けてください...
settings-api-key-save = 保存
settings-api-key-cancel = キャンセル
settings-api-key-how-to = APIキーの取得方法:
settings-api-key-visit = 1. 訪問
settings-api-key-login = 2. ログインして個人用アクセストークンを作成
settings-api-key-configured = ✅ APIキーが設定されました
settings-api-key-change = 変更
settings-api-key-remove = 削除
settings-api-key-secure-message = APIキーは安全に保存され、アプリは完全に機能しています。
settings-api-key-not-configured = ❌ APIキーが設定されていません
settings-api-key-add-prompt = このアプレットを使用するには、WeatherFlow Tempest APIキーを追加してください。
settings-api-key-add-button = APIキーを追加

settings-units-title = 単位
settings-temperature = 温度単位
settings-wind-speed = 風速単位
settings-pressure = 気圧単位
settings-precipitation = 降水量単位
settings-distance = 距離単位

settings-station-title = 気象観測所
settings-station-none = 利用可能な観測所がありません
settings-refresh-interval-title = 更新間隔
settings-location-title = 場所
settings-location-auto-on = 自動位置情報: オン
settings-location-auto-off = 自動位置情報: オフ
settings-location-current = 現在: {$lat}, {$lon}
settings-location-none = 場所が設定されていません
settings-version = バージョン

# Units
unit-celsius = °C
unit-fahrenheit = °F
unit-mps = m/s
unit-kph = km/h
unit-mph = mph
unit-knots = ノット
unit-mb = mb
unit-inhg = inHg
unit-mm = mm
unit-in = in
unit-km = km
unit-mi = mi

# Common
common-loading = 読み込み中...
common-error = エラー
common-refresh = 更新
common-close = 閉じる
common-ok = OK
common-cancel = キャンセル

# About Tab
tab-about = について
about-tagline = Tempest気象観測所からのリアルタイム気象データ
about-version = バージョン
about-description = WebSocket統合と包括的な国際化サポートを備えたTempest気象観測所からのリアルタイム気象データを表示するCOSMICデスクトップパネルアプレット。
about-repository = リポジトリ
about-license = ライセンス

# Lockdown View
lockdown-title = APIキーが必要です
lockdown-message = このアプレットを使用するには、Tempest APIキーを設定してください。
lockdown-configure = APIキーを設定

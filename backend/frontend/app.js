/**
 * Smart Patient Room Monitor - Frontend Application
 * With Activity Reports and Interactive Charts
 */

const CONFIG = {
    wsUrl: `ws://${window.location.host}/ws`,
    apiUrl: `http://${window.location.host}/api`,
    maxDataPoints: 60,
    reconnectDelay: 3000,
};

const state = {
    connected: false,
    ws: null,
    data: [],
    alertSummary: { falls: 0, inactivity: 0 },
    currentDetailType: null,
    currentDetailRange: 15,
    currentReport: null,
};

// ============================================================================
// WebSocket Connection
// ============================================================================

function connectWebSocket() {
    console.log('Connecting to WebSocket:', CONFIG.wsUrl);
    updateConnectionStatus('connecting');
    
    state.ws = new WebSocket(CONFIG.wsUrl);
    
    state.ws.onopen = () => {
        console.log('WebSocket connected');
        state.connected = true;
        updateConnectionStatus('connected');
    };
    
    state.ws.onmessage = (event) => {
        try {
            const message = JSON.parse(event.data);
            handleWebSocketMessage(message);
        } catch (e) {
            console.error('Failed to parse message:', e);
        }
    };
    
    state.ws.onclose = () => {
        console.log('WebSocket disconnected');
        state.connected = false;
        updateConnectionStatus('disconnected');
        setTimeout(connectWebSocket, CONFIG.reconnectDelay);
    };
    
    state.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateConnectionStatus('error');
    };
}

function handleWebSocketMessage(message) {
    switch (message.type) {
        case 'sensorReading':
            handleSensorReading(message);
            break;
        case 'status':
            console.log('Status:', message.message);
            break;
        case 'ping':
            break;
    }
}

function handleSensorReading(reading) {
    state.data.push({
        timestamp: new Date(reading.timestamp),
        temperature: reading.temperature,
        motion: reading.motion,
        soundLevel: reading.soundLevel,
        alert: reading.alert,
    });
    
    if (state.data.length > CONFIG.maxDataPoints) {
        state.data.shift();
    }
    
    updateStats(reading);
    updateCharts();
    updateLastTime();
    addEventToTable(reading);
    
    if (reading.alert) {
        showAlert(reading.alert);
        if (reading.alert === 'FALL_DETECTED') {
            state.alertSummary.falls++;
        } else if (reading.alert === 'INACTIVITY_ALERT') {
            state.alertSummary.inactivity++;
        }
        updateAlertCounts();
    }
}

// ============================================================================
// UI Updates
// ============================================================================

function updateConnectionStatus(status) {
    const statusEl = document.getElementById('connectionStatus');
    const dot = statusEl.querySelector('.status-dot');
    const text = statusEl.querySelector('.status-text');
    
    dot.className = 'status-dot';
    
    switch (status) {
        case 'connected':
            dot.classList.add('connected');
            text.textContent = 'Connected';
            break;
        case 'connecting':
            text.textContent = 'Connecting...';
            break;
        case 'disconnected':
            text.textContent = 'Reconnecting...';
            break;
        case 'error':
            dot.classList.add('error');
            text.textContent = 'Connection Error';
            break;
    }
}

function updateStats(reading) {
    document.getElementById('tempValue').textContent = reading.temperature.toFixed(1);
    
    const motionValue = document.getElementById('motionValue');
    const motionStatus = document.getElementById('motionStatus');
    
    if (reading.motion) {
        motionValue.textContent = 'YES';
        motionStatus.textContent = 'Movement detected';
    } else {
        motionValue.textContent = 'NO';
        motionStatus.textContent = 'No movement';
    }
    
    document.getElementById('soundValue').textContent = reading.soundLevel;
    const soundBar = document.getElementById('soundBar');
    const soundPercent = Math.min(100, (reading.soundLevel / 400) * 100);
    soundBar.style.width = `${soundPercent}%`;
}

function updateAlertCounts() {
    const total = state.alertSummary.falls + state.alertSummary.inactivity;
    document.getElementById('alertCount').textContent = total;
    document.getElementById('fallCount').textContent = state.alertSummary.falls;
    document.getElementById('inactiveCount').textContent = state.alertSummary.inactivity;
}

function updateLastTime() {
    const now = new Date();
    document.getElementById('lastUpdate').textContent = `Last update: ${now.toLocaleTimeString()}`;
}

// ============================================================================
// Audio Alert System
// ============================================================================

let audioEnabled = true;
let lastAlertTime = 0;
const ALERT_COOLDOWN = 10000; // 10 seconds between alerts

// Create alert tone using Web Audio API (works offline)
function createAlertTone() {
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();
    
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    
    oscillator.frequency.value = 880; // A5 note - calm but attention-getting
    oscillator.type = 'sine';
    
    gainNode.gain.setValueAtTime(0, audioContext.currentTime);
    gainNode.gain.linearRampToValueAtTime(0.3, audioContext.currentTime + 0.1);
    gainNode.gain.linearRampToValueAtTime(0, audioContext.currentTime + 0.5);
    
    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.5);
    
    // Second beep
    setTimeout(() => {
        const osc2 = audioContext.createOscillator();
        const gain2 = audioContext.createGain();
        
        osc2.connect(gain2);
        gain2.connect(audioContext.destination);
        
        osc2.frequency.value = 880;
        osc2.type = 'sine';
        
        gain2.gain.setValueAtTime(0, audioContext.currentTime);
        gain2.gain.linearRampToValueAtTime(0.3, audioContext.currentTime + 0.1);
        gain2.gain.linearRampToValueAtTime(0, audioContext.currentTime + 0.5);
        
        osc2.start(audioContext.currentTime);
        osc2.stop(audioContext.currentTime + 0.5);
    }, 600);
    
    // Third beep
    setTimeout(() => {
        const osc3 = audioContext.createOscillator();
        const gain3 = audioContext.createGain();
        
        osc3.connect(gain3);
        gain3.connect(audioContext.destination);
        
        osc3.frequency.value = 1046.5; // C6 - slightly higher for attention
        osc3.type = 'sine';
        
        gain3.gain.setValueAtTime(0, audioContext.currentTime);
        gain3.gain.linearRampToValueAtTime(0.3, audioContext.currentTime + 0.1);
        gain3.gain.linearRampToValueAtTime(0, audioContext.currentTime + 0.8);
        
        osc3.start(audioContext.currentTime);
        osc3.stop(audioContext.currentTime + 0.8);
    }, 1200);
}

function speakAlert(message) {
    if (!('speechSynthesis' in window)) return;
    
    // Cancel any ongoing speech
    speechSynthesis.cancel();
    
    const utterance = new SpeechSynthesisUtterance(message);
    utterance.rate = 0.85;    // Slower, calmer pace
    utterance.pitch = 0.95;   // Slightly lower pitch for calmness
    utterance.volume = 1;
    
    // Try to find a professional-sounding voice
    const voices = speechSynthesis.getVoices();
    const preferredVoice = voices.find(v => 
        v.name.includes('Google UK English Female') ||
        v.name.includes('Microsoft Zira') ||
        v.name.includes('Samantha') ||
        v.name.includes('Karen') ||
        v.lang.startsWith('en')
    );
    
    if (preferredVoice) {
        utterance.voice = preferredVoice;
    }
    
    speechSynthesis.speak(utterance);
}

function playFallAlert() {
    if (!audioEnabled) return;
    
    const now = Date.now();
    if (now - lastAlertTime < ALERT_COOLDOWN) return;
    lastAlertTime = now;
    
    // Play tone first
    createAlertTone();
    
    // Then speak after tone finishes
    setTimeout(() => {
        speakAlert('Attention. Possible fall detected. Please check the patient.');
    }, 2000);
}

function playInactivityAlert() {
    if (!audioEnabled) return;
    
    const now = Date.now();
    if (now - lastAlertTime < ALERT_COOLDOWN) return;
    lastAlertTime = now;
    
    // Just play tone, no voice for inactivity
    createAlertTone();
}

function toggleAudio() {
    audioEnabled = !audioEnabled;
    const btn = document.getElementById('audioToggleBtn');
    if (btn) {
        btn.innerHTML = audioEnabled 
            ? '🔊 Sound On' 
            : '🔇 Sound Off';
        btn.classList.toggle('muted', !audioEnabled);
    }
}

// Load voices when available
if ('speechSynthesis' in window) {
    speechSynthesis.onvoiceschanged = () => {
        speechSynthesis.getVoices();
    };
}

function showAlert(alertType) {
    const banner = document.getElementById('alertBanner');
    const message = document.getElementById('alertMessage');
    
    if (alertType === 'FALL_DETECTED') {
        message.textContent = '⚠️ POSSIBLE FALL DETECTED - Check patient immediately!';
        playFallAlert();
    } else if (alertType === 'INACTIVITY_ALERT') {
        message.textContent = '⚠️ Patient inactivity detected - No movement for extended period';
        playInactivityAlert();
    }
    
    banner.classList.remove('hidden');
}

function dismissAlert() {
    document.getElementById('alertBanner').classList.add('hidden');
}

// ============================================================================
// Settings Management
// ============================================================================

const settings = {
    inactivityThreshold: 300, // seconds (5 minutes default)
    soundThreshold: 150,
};

function openSettingsModal() {
    document.getElementById('settingsModal').classList.add('active');
    loadCurrentSettings();
}

function closeSettingsModal() {
    document.getElementById('settingsModal').classList.remove('active');
}

async function loadCurrentSettings() {
    try {
        const response = await fetch(`${CONFIG.apiUrl}/settings`);
        if (response.ok) {
            const data = await response.json();
            // Rust backend returns snake_case
            settings.inactivityThreshold = data.inactivity_seconds || 300;
            settings.soundThreshold = data.sound_threshold || 150;
            
            // Update UI
            const minutes = Math.round(settings.inactivityThreshold / 60);
            document.getElementById('thresholdSlider').value = minutes;
            document.getElementById('thresholdValue').textContent = minutes;
            
            document.getElementById('soundThresholdSlider').value = settings.soundThreshold;
            document.getElementById('soundThresholdValue').textContent = settings.soundThreshold;
        }
    } catch (error) {
        console.log('Using default settings');
    }
}

function updateThresholdDisplay(value) {
    document.getElementById('thresholdValue').textContent = value;
}

function updateSoundThresholdDisplay(value) {
    document.getElementById('soundThresholdValue').textContent = value;
}

function setThreshold(minutes) {
    document.getElementById('thresholdSlider').value = minutes;
    document.getElementById('thresholdValue').textContent = minutes;
}

async function saveSettings() {
    const inactivityMinutes = parseInt(document.getElementById('thresholdSlider').value);
    const soundThreshold = parseInt(document.getElementById('soundThresholdSlider').value);
    
    const statusEl = document.getElementById('settingsStatus');
    statusEl.className = 'settings-status saving';
    statusEl.innerHTML = '<div class="status-icon">⏳</div><span>Saving settings...</span>';
    
    try {
        const response = await fetch(`${CONFIG.apiUrl}/settings`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                inactivity_seconds: inactivityMinutes * 60,
                sound_threshold: soundThreshold,
            }),
        });
        
        if (response.ok) {
            settings.inactivityThreshold = inactivityMinutes * 60;
            settings.soundThreshold = soundThreshold;
            
            statusEl.className = 'settings-status';
            statusEl.innerHTML = '<div class="status-icon">✓</div><span>Settings saved successfully!</span>';
            
            // Close modal after a short delay
            setTimeout(() => {
                closeSettingsModal();
            }, 1500);
        } else {
            throw new Error('Failed to save');
        }
    } catch (error) {
        console.error('Failed to save settings:', error);
        statusEl.className = 'settings-status error';
        statusEl.innerHTML = '<div class="status-icon">✕</div><span>Failed to save settings. Please try again.</span>';
    }
}

// ============================================================================
// D3.js Charts
// ============================================================================

let realtimeChart = null;
let motionChart = null;

function initCharts() {
    initRealtimeChart();
    initMotionChart();
}

function initRealtimeChart() {
    const container = document.getElementById('realtimeChart');
    const width = container.clientWidth;
    const height = container.clientHeight;
    const margin = { top: 20, right: 50, bottom: 30, left: 50 };
    
    container.innerHTML = '';
    
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const defs = svg.append('defs');
    
    const tempGradient = defs.append('linearGradient')
        .attr('id', 'gradient-temp')
        .attr('x1', '0%').attr('y1', '0%')
        .attr('x2', '0%').attr('y2', '100%');
    tempGradient.append('stop').attr('offset', '0%').attr('stop-color', '#f97316').attr('stop-opacity', 0.3);
    tempGradient.append('stop').attr('offset', '100%').attr('stop-color', '#f97316').attr('stop-opacity', 0);
    
    const soundGradient = defs.append('linearGradient')
        .attr('id', 'gradient-sound')
        .attr('x1', '0%').attr('y1', '0%')
        .attr('x2', '0%').attr('y2', '100%');
    soundGradient.append('stop').attr('offset', '0%').attr('stop-color', '#3b82f6').attr('stop-opacity', 0.3);
    soundGradient.append('stop').attr('offset', '100%').attr('stop-color', '#3b82f6').attr('stop-opacity', 0);
    
    const g = svg.append('g').attr('transform', `translate(${margin.left},${margin.top})`);
    
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;
    
    const xScale = d3.scaleTime().range([0, innerWidth]);
    const yScaleTemp = d3.scaleLinear().range([innerHeight, 0]).domain([15, 35]);
    const yScaleSound = d3.scaleLinear().range([innerHeight, 0]).domain([0, 400]);
    
    const xAxis = g.append('g').attr('class', 'axis x-axis').attr('transform', `translate(0,${innerHeight})`);
    const yAxisTemp = g.append('g').attr('class', 'axis y-axis-temp');
    const yAxisSound = g.append('g').attr('class', 'axis y-axis-sound').attr('transform', `translate(${innerWidth},0)`);
    const grid = g.append('g').attr('class', 'grid');
    
    const areaTemp = d3.area()
        .x(d => xScale(d.timestamp))
        .y0(innerHeight)
        .y1(d => yScaleTemp(d.temperature))
        .curve(d3.curveMonotoneX);
    
    const areaSound = d3.area()
        .x(d => xScale(d.timestamp))
        .y0(innerHeight)
        .y1(d => yScaleSound(d.soundLevel))
        .curve(d3.curveMonotoneX);
    
    const lineTemp = d3.line()
        .x(d => xScale(d.timestamp))
        .y(d => yScaleTemp(d.temperature))
        .curve(d3.curveMonotoneX);
    
    const lineSound = d3.line()
        .x(d => xScale(d.timestamp))
        .y(d => yScaleSound(d.soundLevel))
        .curve(d3.curveMonotoneX);
    
    const areaTempPath = g.append('path').attr('class', 'area-temp');
    const areaSoundPath = g.append('path').attr('class', 'area-sound');
    const lineTempPath = g.append('path').attr('class', 'line-temp');
    const lineSoundPath = g.append('path').attr('class', 'line-sound');
    
    realtimeChart = {
        svg, g, xScale, yScaleTemp, yScaleSound,
        xAxis, yAxisTemp, yAxisSound, grid,
        areaTemp, areaSound, lineTemp, lineSound,
        areaTempPath, areaSoundPath, lineTempPath, lineSoundPath,
        innerWidth, innerHeight
    };
}

function initMotionChart() {
    const container = document.getElementById('motionChart');
    const width = container.clientWidth;
    const height = container.clientHeight;
    const margin = { top: 10, right: 10, bottom: 30, left: 10 };
    
    container.innerHTML = '';
    
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const g = svg.append('g').attr('transform', `translate(${margin.left},${margin.top})`);
    
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;
    
    motionChart = { svg, g, innerWidth, innerHeight };
}

function updateCharts() {
    if (state.data.length < 2) return;
    updateRealtimeChart();
    updateMotionChart();
}

function updateRealtimeChart() {
    const chart = realtimeChart;
    if (!chart) return;
    
    const data = state.data;
    
    chart.xScale.domain(d3.extent(data, d => d.timestamp));
    
    chart.xAxis.call(d3.axisBottom(chart.xScale).ticks(5).tickFormat(d3.timeFormat('%H:%M:%S')));
    chart.yAxisTemp.call(d3.axisLeft(chart.yScaleTemp).ticks(5).tickFormat(d => `${d}°C`));
    chart.yAxisSound.call(d3.axisRight(chart.yScaleSound).ticks(5));
    
    chart.grid.call(d3.axisLeft(chart.yScaleTemp).ticks(5).tickSize(-chart.innerWidth).tickFormat(''));
    
    chart.areaTempPath.datum(data).attr('d', chart.areaTemp);
    chart.areaSoundPath.datum(data).attr('d', chart.areaSound);
    chart.lineTempPath.datum(data).attr('d', chart.lineTemp);
    chart.lineSoundPath.datum(data).attr('d', chart.lineSound);
}

function updateMotionChart() {
    const chart = motionChart;
    if (!chart) return;
    
    const data = state.data.slice(-60);
    const cellWidth = Math.max(8, chart.innerWidth / 60);
    const cellHeight = chart.innerHeight;
    
    const cells = chart.g.selectAll('.motion-cell').data(data, (d, i) => i);
    
    cells.enter()
        .append('rect')
        .attr('class', 'motion-cell')
        .attr('y', 0)
        .attr('height', cellHeight)
        .attr('width', cellWidth - 2)
        .merge(cells)
        .attr('x', (d, i) => i * cellWidth)
        .attr('class', d => {
            if (d.alert) return 'motion-cell alert';
            return d.motion ? 'motion-cell active' : 'motion-cell inactive';
        });
    
    cells.exit().remove();
}

// ============================================================================
// Report Modal
// ============================================================================

function openReportModal() {
    document.getElementById('reportModal').classList.add('active');
    document.getElementById('reportDate').valueAsDate = new Date();
}

function closeReportModal() {
    document.getElementById('reportModal').classList.remove('active');
}

function setPreset(preset) {
    const startTime = document.getElementById('reportStartTime');
    const endTime = document.getElementById('reportEndTime');
    
    switch (preset) {
        case 'night':
            startTime.value = '22:00';
            endTime.value = '06:00';
            break;
        case 'morning':
            startTime.value = '06:00';
            endTime.value = '12:00';
            break;
        case 'afternoon':
            startTime.value = '12:00';
            endTime.value = '18:00';
            break;
        case 'evening':
            startTime.value = '18:00';
            endTime.value = '22:00';
            break;
        case '24h':
            startTime.value = '00:00';
            endTime.value = '23:59';
            break;
    }
}

async function generateReport() {
    const date = document.getElementById('reportDate').value;
    const startTime = document.getElementById('reportStartTime').value;
    const endTime = document.getElementById('reportEndTime').value;
    
    if (!date) {
        alert('Please select a date');
        return;
    }
    
    const startHour = parseInt(startTime.split(':')[0]);
    const endHour = parseInt(endTime.split(':')[0]);
    
    const reportContent = document.getElementById('reportContent');
    reportContent.innerHTML = '<div class="report-placeholder"><p>Generating report...</p></div>';
    
    try {
        const response = await fetch(
            `${CONFIG.apiUrl}/activity/sleep?date=${date}&start_hour=${startHour}&end_hour=${endHour}`
        );
        const data = await response.json();
        
        state.currentReport = data;
        renderReport(data, date, startTime, endTime);
        document.getElementById('reportFooter').style.display = 'flex';
        
    } catch (error) {
        console.error('Failed to generate report:', error);
        reportContent.innerHTML = `
            <div class="report-placeholder">
                <p>Failed to generate report. Please try again.</p>
            </div>
        `;
    }
}

function renderReport(data, date, startTime, endTime) {
    const activityClass = data.activityScore < 20 ? 'good' : 
                          data.activityScore < 40 ? 'good' :
                          data.activityScore < 60 ? 'warning' : 'danger';
    
    const reportContent = document.getElementById('reportContent');
    reportContent.innerHTML = `
        <div class="report-results">
            <div class="report-summary">
                <div class="summary-item">
                    <div class="summary-value ${activityClass}">${data.activityScore.toFixed(1)}%</div>
                    <div class="summary-label">Activity Score</div>
                </div>
                <div class="summary-item">
                    <div class="summary-value">${data.totalReadings}</div>
                    <div class="summary-label">Total Readings</div>
                </div>
                <div class="summary-item">
                    <div class="summary-value">${data.motionReadings}</div>
                    <div class="summary-label">Motion Events</div>
                </div>
                <div class="summary-item">
                    <div class="summary-value ${data.fallAlerts > 0 ? 'danger' : ''}">${data.fallAlerts}</div>
                    <div class="summary-label">Fall Alerts</div>
                </div>
            </div>
            
            <div class="report-chart">
                <h3>Activity Level Assessment</h3>
                <div style="display: flex; align-items: center; gap: 16px; margin-bottom: 16px;">
                    <span class="activity-badge ${data.activityLevel}">${formatActivityLevel(data.activityLevel)}</span>
                    <span style="color: var(--color-text-secondary); font-size: 0.875rem;">
                        ${getActivityDescription(data.activityLevel)}
                    </span>
                </div>
                <div class="report-chart-container" id="reportChartContainer"></div>
            </div>
            
            <div class="report-details">
                <div class="detail-card">
                    <h4>📊 Statistics</h4>
                    <div class="detail-row">
                        <span class="detail-label">Period</span>
                        <span class="detail-value">${date} ${startTime} - ${endTime}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Avg Temperature</span>
                        <span class="detail-value">${data.avgTemperature.toFixed(1)}°C</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Avg Sound Level</span>
                        <span class="detail-value">${data.avgSoundLevel.toFixed(0)}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Max Sound Level</span>
                        <span class="detail-value">${data.maxSoundLevel}</span>
                    </div>
                </div>
                
                <div class="detail-card">
                    <h4>😴 Sleep Analysis</h4>
                    <div class="detail-row">
                        <span class="detail-label">Longest Still Period</span>
                        <span class="detail-value">${data.longestStillPeriodMins} min</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Motion Percentage</span>
                        <span class="detail-value">${data.activityScore.toFixed(1)}%</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Rest Quality</span>
                        <span class="detail-value">${getRestQuality(data.activityScore)}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Recommendation</span>
                        <span class="detail-value">${getRecommendation(data.activityLevel)}</span>
                    </div>
                </div>
            </div>
        </div>
    `;
    
    // Draw activity gauge chart
    setTimeout(() => drawActivityGauge(data.activityScore), 100);
}

function formatActivityLevel(level) {
    const labels = {
        'deep_sleep': '😴 Deep Sleep',
        'light_sleep': '💤 Light Sleep',
        'restless': '😟 Restless',
        'active': '🚶 Active'
    };
    return labels[level] || level;
}

function getActivityDescription(level) {
    const descriptions = {
        'deep_sleep': 'Patient had minimal movement - good quality rest',
        'light_sleep': 'Some movement detected - acceptable rest quality',
        'restless': 'Frequent movement - consider checking on patient',
        'active': 'High activity - patient may not be resting'
    };
    return descriptions[level] || '';
}

function getRestQuality(score) {
    if (score < 20) return 'Excellent';
    if (score < 40) return 'Good';
    if (score < 60) return 'Fair';
    return 'Poor';
}

function getRecommendation(level) {
    const recommendations = {
        'deep_sleep': 'No action needed',
        'light_sleep': 'Monitor continued',
        'restless': 'Check comfort',
        'active': 'Assess patient'
    };
    return recommendations[level] || 'Monitor';
}

function drawActivityGauge(score) {
    const container = document.getElementById('reportChartContainer');
    if (!container) return;
    
    const width = container.clientWidth;
    const height = 180;
    
    container.innerHTML = '';
    
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const centerX = width / 2;
    const centerY = height - 20;
    const radius = Math.min(width / 2, height) - 30;
    
    // Background arc
    const bgArc = d3.arc()
        .innerRadius(radius - 20)
        .outerRadius(radius)
        .startAngle(-Math.PI / 2)
        .endAngle(Math.PI / 2);
    
    svg.append('path')
        .attr('d', bgArc)
        .attr('transform', `translate(${centerX}, ${centerY})`)
        .attr('fill', '#1e293b');
    
    // Score arc
    const scoreAngle = -Math.PI / 2 + (Math.PI * score / 100);
    const scoreArc = d3.arc()
        .innerRadius(radius - 20)
        .outerRadius(radius)
        .startAngle(-Math.PI / 2)
        .endAngle(scoreAngle);
    
    const color = score < 20 ? '#22c55e' : 
                  score < 40 ? '#3b82f6' :
                  score < 60 ? '#f59e0b' : '#ef4444';
    
    svg.append('path')
        .attr('d', scoreArc)
        .attr('transform', `translate(${centerX}, ${centerY})`)
        .attr('fill', color);
    
    // Labels
    const labels = [
        { angle: -Math.PI / 2, text: '0%' },
        { angle: -Math.PI / 4, text: '25%' },
        { angle: 0, text: '50%' },
        { angle: Math.PI / 4, text: '75%' },
        { angle: Math.PI / 2, text: '100%' }
    ];
    
    labels.forEach(l => {
        const x = centerX + (radius + 15) * Math.cos(l.angle - Math.PI / 2);
        const y = centerY + (radius + 15) * Math.sin(l.angle - Math.PI / 2);
        
        svg.append('text')
            .attr('x', x)
            .attr('y', y)
            .attr('text-anchor', 'middle')
            .attr('fill', '#64748b')
            .attr('font-size', '10px')
            .text(l.text);
    });
    
    // Center text
    svg.append('text')
        .attr('x', centerX)
        .attr('y', centerY - 20)
        .attr('text-anchor', 'middle')
        .attr('fill', color)
        .attr('font-size', '32px')
        .attr('font-weight', '700')
        .attr('font-family', 'JetBrains Mono')
        .text(`${score.toFixed(1)}%`);
    
    svg.append('text')
        .attr('x', centerX)
        .attr('y', centerY)
        .attr('text-anchor', 'middle')
        .attr('fill', '#94a3b8')
        .attr('font-size', '12px')
        .text('Activity Score');
}

function downloadReport() {
    if (!state.currentReport) return;
    
    const data = state.currentReport;
    const date = document.getElementById('reportDate').value;
    const startTime = document.getElementById('reportStartTime').value;
    const endTime = document.getElementById('reportEndTime').value;
    
    // Create text report
    const report = `
SMART PATIENT ROOM MONITOR
Activity Report
================================

Report Period: ${date} ${startTime} - ${endTime}
Generated: ${new Date().toLocaleString()}

SUMMARY
-------
Activity Score: ${data.activityScore.toFixed(1)}%
Activity Level: ${formatActivityLevel(data.activityLevel)}
Total Readings: ${data.totalReadings}
Motion Events: ${data.motionReadings}
Fall Alerts: ${data.fallAlerts}

STATISTICS
----------
Average Temperature: ${data.avgTemperature.toFixed(1)}°C
Average Sound Level: ${data.avgSoundLevel.toFixed(0)}
Maximum Sound Level: ${data.maxSoundLevel}
Longest Still Period: ${data.longestStillPeriodMins} minutes

ASSESSMENT
----------
Rest Quality: ${getRestQuality(data.activityScore)}
Recommendation: ${getRecommendation(data.activityLevel)}

${getActivityDescription(data.activityLevel)}

================================
Report generated by Smart Patient Room Monitor
FHIR Compliant Healthcare Monitoring System
    `.trim();
    
    // Download as text file
    const blob = new Blob([report], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `activity_report_${date}_${startTime.replace(':', '')}-${endTime.replace(':', '')}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// ============================================================================
// Sensor Detail Modal
// ============================================================================

function openDetailModal(type) {
    if (type === 'alerts') return; // Alerts card is not interactive
    
    state.currentDetailType = type;
    state.currentDetailRange = 15;
    
    const titles = {
        'temperature': '🌡️ Temperature Details',
        'motion': '🚶 Motion Activity Analysis',
        'sound': '🔊 Sound Level Details'
    };
    
    document.getElementById('detailModalTitle').textContent = titles[type];
    document.getElementById('detailModal').classList.add('active');
    
    // Reset time buttons
    document.querySelectorAll('.time-btn').forEach(btn => btn.classList.remove('active'));
    document.querySelector('.time-btn').classList.add('active');
    
    loadDetailData(type, 15);
}

function closeDetailModal() {
    document.getElementById('detailModal').classList.remove('active');
}

function setDetailRange(minutes) {
    state.currentDetailRange = minutes;
    
    // Update button states
    document.querySelectorAll('.time-btn').forEach(btn => {
        btn.classList.remove('active');
        if (parseInt(btn.textContent) === minutes || 
            (btn.textContent.includes('hour') && minutes === parseInt(btn.textContent) * 60)) {
            btn.classList.add('active');
        }
    });
    
    // Find and activate the correct button
    document.querySelectorAll('.time-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');
    
    loadDetailData(state.currentDetailType, minutes);
}

async function loadDetailData(type, minutes) {
    try {
        // Fetch activity analysis
        const response = await fetch(`${CONFIG.apiUrl}/activity/period?minutes=${minutes}`);
        const data = await response.json();
        
        renderDetailStats(type, data);
        
        // Fetch historical observations for the chart
        const now = new Date();
        const end = now.toISOString();
        const start = new Date(now.getTime() - minutes * 60 * 1000).toISOString();
        
        const historyResponse = await fetch(`${CONFIG.apiUrl}/observations?start=${start}&end=${end}&_count=100`);
        const historyData = await historyResponse.json();
        
        renderDetailChart(type, minutes, historyData);
        
    } catch (error) {
        console.error('Failed to load detail data:', error);
    }
}

function renderDetailStats(type, data) {
    const statsContainer = document.getElementById('detailStats');
    
    let statsHtml = '';
    
    switch (type) {
        case 'temperature':
            statsHtml = `
                <div class="detail-stat-item">
                    <div class="detail-stat-value" style="color: #f97316">${data.avgTemperature.toFixed(1)}°C</div>
                    <div class="detail-stat-label">Average</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.totalReadings}</div>
                    <div class="detail-stat-label">Readings</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${getTemperatureStatus(data.avgTemperature)}</div>
                    <div class="detail-stat-label">Status</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.avgTemperature > 25 ? '⚠️' : '✓'}</div>
                    <div class="detail-stat-label">Comfort</div>
                </div>
            `;
            break;
            
        case 'motion':
            statsHtml = `
                <div class="detail-stat-item">
                    <div class="detail-stat-value" style="color: #22c55e">${data.activityScore.toFixed(1)}%</div>
                    <div class="detail-stat-label">Activity</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.motionReadings}</div>
                    <div class="detail-stat-label">Motion Events</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.longestStillPeriodMins}m</div>
                    <div class="detail-stat-label">Longest Still</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${formatActivityLevel(data.activityLevel).split(' ')[0]}</div>
                    <div class="detail-stat-label">Level</div>
                </div>
            `;
            break;
            
        case 'sound':
            statsHtml = `
                <div class="detail-stat-item">
                    <div class="detail-stat-value" style="color: #3b82f6">${data.avgSoundLevel.toFixed(0)}</div>
                    <div class="detail-stat-label">Average</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.maxSoundLevel}</div>
                    <div class="detail-stat-label">Maximum</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${data.fallAlerts}</div>
                    <div class="detail-stat-label">Fall Alerts</div>
                </div>
                <div class="detail-stat-item">
                    <div class="detail-stat-value">${getSoundStatus(data.avgSoundLevel)}</div>
                    <div class="detail-stat-label">Environment</div>
                </div>
            `;
            break;
    }
    
    statsContainer.innerHTML = statsHtml;
}

function getTemperatureStatus(temp) {
    if (temp < 18) return 'Cold';
    if (temp < 22) return 'Cool';
    if (temp < 26) return 'Normal';
    if (temp < 30) return 'Warm';
    return 'Hot';
}

function getSoundStatus(level) {
    if (level < 30) return 'Quiet';
    if (level < 60) return 'Normal';
    if (level < 100) return 'Moderate';
    return 'Loud';
}

function renderDetailChart(type, minutes, historyData) {
    const container = document.getElementById('detailChart');
    
    // Parse FHIR bundle data
    let data = [];
    if (historyData && historyData.entry && historyData.entry.length > 0) {
        data = historyData.entry.map(entry => {
            const obs = entry.resource;
            const reading = {
                timestamp: new Date(obs.effectiveDateTime),
                temperature: null,
                motion: false,
                soundLevel: 0,
                alert: null
            };
            
            if (obs.component) {
                obs.component.forEach(comp => {
                    const code = comp.code?.coding?.[0]?.code;
                    if (code === '8310-5') {
                        reading.temperature = comp.valueQuantity?.value;
                    } else if (code === '52821000') {
                        reading.motion = comp.valueBoolean;
                    } else if (code === '89020-2') {
                        reading.soundLevel = comp.valueInteger;
                    } else if (code === 'AA') {
                        reading.alert = comp.valueString;
                    }
                });
            }
            return reading;
        }).reverse(); // Reverse to show chronological order
    }
    
    // Fall back to WebSocket data if no history available
    if (data.length < 2) {
        data = state.data.slice(-Math.min(state.data.length, minutes));
    }
    
    if (data.length < 2) {
        container.innerHTML = '<p style="text-align: center; color: #64748b; padding: 40px;">Not enough data yet</p>';
        return;
    }
    
    const width = container.clientWidth;
    const height = 300;
    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    
    container.innerHTML = '';
    
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const g = svg.append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);
    
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;
    
    const xScale = d3.scaleTime()
        .domain(d3.extent(data, d => d.timestamp))
        .range([0, innerWidth]);
    
    let yScale, lineGenerator, areaGenerator, color;
    
    switch (type) {
        case 'temperature':
            yScale = d3.scaleLinear().domain([15, 35]).range([innerHeight, 0]);
            lineGenerator = d3.line()
                .x(d => xScale(d.timestamp))
                .y(d => yScale(d.temperature))
                .curve(d3.curveMonotoneX);
            areaGenerator = d3.area()
                .x(d => xScale(d.timestamp))
                .y0(innerHeight)
                .y1(d => yScale(d.temperature))
                .curve(d3.curveMonotoneX);
            color = '#f97316';
            break;
            
        case 'sound':
            yScale = d3.scaleLinear().domain([0, 400]).range([innerHeight, 0]);
            lineGenerator = d3.line()
                .x(d => xScale(d.timestamp))
                .y(d => yScale(d.soundLevel))
                .curve(d3.curveMonotoneX);
            areaGenerator = d3.area()
                .x(d => xScale(d.timestamp))
                .y0(innerHeight)
                .y1(d => yScale(d.soundLevel))
                .curve(d3.curveMonotoneX);
            color = '#3b82f6';
            break;
            
        case 'motion':
            // For motion, show a bar chart
            yScale = d3.scaleLinear().domain([0, 1]).range([innerHeight, 0]);
            
            const barWidth = Math.max(2, innerWidth / data.length - 1);
            
            g.selectAll('.motion-bar')
                .data(data)
                .enter()
                .append('rect')
                .attr('class', 'motion-bar')
                .attr('x', d => xScale(d.timestamp) - barWidth / 2)
                .attr('y', d => d.motion ? yScale(1) : yScale(0))
                .attr('width', barWidth)
                .attr('height', d => d.motion ? innerHeight : 0)
                .attr('fill', d => d.alert ? '#ef4444' : '#22c55e')
                .attr('opacity', 0.7);
            
            // Add axis
            g.append('g')
                .attr('class', 'axis')
                .attr('transform', `translate(0,${innerHeight})`)
                .call(d3.axisBottom(xScale).ticks(5).tickFormat(d3.timeFormat('%H:%M')));
            
            g.append('g')
                .attr('class', 'axis')
                .call(d3.axisLeft(yScale).ticks(2).tickFormat(d => d ? 'Motion' : 'Still'));
            
            return;
    }
    
    // Add gradient
    const gradient = svg.append('defs')
        .append('linearGradient')
        .attr('id', `gradient-detail-${type}`)
        .attr('x1', '0%').attr('y1', '0%')
        .attr('x2', '0%').attr('y2', '100%');
    
    gradient.append('stop')
        .attr('offset', '0%')
        .attr('stop-color', color)
        .attr('stop-opacity', 0.3);
    
    gradient.append('stop')
        .attr('offset', '100%')
        .attr('stop-color', color)
        .attr('stop-opacity', 0);
    
    // Draw area
    g.append('path')
        .datum(data)
        .attr('fill', `url(#gradient-detail-${type})`)
        .attr('d', areaGenerator);
    
    // Draw line
    g.append('path')
        .datum(data)
        .attr('fill', 'none')
        .attr('stroke', color)
        .attr('stroke-width', 2)
        .attr('d', lineGenerator);
    
    // Add axes
    g.append('g')
        .attr('class', 'axis')
        .attr('transform', `translate(0,${innerHeight})`)
        .call(d3.axisBottom(xScale).ticks(5).tickFormat(d3.timeFormat('%H:%M')));
    
    g.append('g')
        .attr('class', 'axis')
        .call(d3.axisLeft(yScale).ticks(5));
    
    // Add grid
    g.append('g')
        .attr('class', 'grid')
        .call(d3.axisLeft(yScale).ticks(5).tickSize(-innerWidth).tickFormat(''));
}

// ============================================================================
// API Calls
// ============================================================================

async function fetchHistory() {
    try {
        const response = await fetch(`${CONFIG.apiUrl}/observations?_count=20`);
        const bundle = await response.json();
        
        if (bundle.entry && bundle.entry.length > 0) {
            updateEventsTable(bundle.entry);
        }
    } catch (error) {
        console.error('Failed to fetch history:', error);
    }
}

async function fetchSummary() {
    try {
        const response = await fetch(`${CONFIG.apiUrl}/summary`);
        const summary = await response.json();
        
        state.alertSummary.falls = summary.fallAlerts || 0;
        state.alertSummary.inactivity = summary.inactivityAlerts || 0;
        updateAlertCounts();
    } catch (error) {
        console.error('Failed to fetch summary:', error);
    }
}

function addEventToTable(reading) {
    const tbody = document.getElementById('eventsBody');
    
    // Remove "loading" or "no data" placeholder if it exists
    if (tbody.querySelector('.loading-cell')) {
        tbody.innerHTML = '';
    }
    
    const time = new Date(reading.timestamp).toLocaleString();
    const temp = reading.temperature?.toFixed(1) ?? '--';
    const motion = reading.motion ? 'Yes' : 'No';
    const sound = reading.soundLevel ?? '--';
    let alertStatus = 'normal';
    
    if (reading.alert) {
        if (reading.alert === 'FALL_DETECTED') alertStatus = 'fall';
        else if (reading.alert === 'INACTIVITY_ALERT') alertStatus = 'inactivity';
    }
    
    const newRow = document.createElement('tr');
    newRow.innerHTML = `
        <td>${time}</td>
        <td>${temp}°C</td>
        <td>${motion}</td>
        <td>${sound}</td>
        <td><span class="status-badge ${alertStatus}">${alertStatus}</span></td>
    `;
    
    // Insert at the top of the table (most recent first)
    tbody.insertBefore(newRow, tbody.firstChild);
    
    // Keep only the most recent 20 rows
    const maxRows = 20;
    while (tbody.children.length > maxRows) {
        tbody.removeChild(tbody.lastChild);
    }
}

function updateEventsTable(entries) {
    const tbody = document.getElementById('eventsBody');
    
    if (!entries || entries.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" class="loading-cell">No data recorded yet</td></tr>';
        return;
    }
    
    tbody.innerHTML = entries.map(entry => {
        const obs = entry.resource;
        const time = new Date(obs.effectiveDateTime).toLocaleString();
        
        let temp = '--', motion = '--', sound = '--';
        let alertStatus = 'normal';
        
        if (obs.component) {
            obs.component.forEach(comp => {
                const code = comp.code?.coding?.[0]?.code;
                if (code === '8310-5') {
                    temp = comp.valueQuantity?.value?.toFixed(1) || '--';
                } else if (code === '52821000') {
                    motion = comp.valueBoolean ? 'Yes' : 'No';
                } else if (code === '89020-2') {
                    sound = comp.valueInteger ?? '--';
                } else if (code === 'AA' && comp.valueString) {
                    if (comp.valueString === 'FALL_DETECTED') alertStatus = 'fall';
                    else if (comp.valueString === 'INACTIVITY_ALERT') alertStatus = 'inactivity';
                }
            });
        }
        
        return `
            <tr>
                <td>${time}</td>
                <td>${temp}°C</td>
                <td>${motion}</td>
                <td>${sound}</td>
                <td><span class="status-badge ${alertStatus}">${alertStatus}</span></td>
            </tr>
        `;
    }).join('');
}

// ============================================================================
// Initialization
// ============================================================================

function init() {
    console.log('Initializing Smart Patient Monitor...');
    initCharts();
    connectWebSocket();
    fetchHistory();
    fetchSummary();
    
    window.addEventListener('resize', () => {
        initCharts();
        updateCharts();
    });
}

document.addEventListener('DOMContentLoaded', init);
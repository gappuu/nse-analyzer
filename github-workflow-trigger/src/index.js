// src/index.js
// Market schedule configuration
const marketSchedule = {
  normalHours: {
    start: '09:15',
    end: '15:30',
  },
  
  dataUpdateTimes: [
    '10:40', '11:40', '12:40', '13:40', '14:40', '15:40',
  ],
  
  holidays: [
    { date: "2026-01-15", name: "Municipal Corporation Election - Maharashtra", type: 'holiday' },
    { date: "2026-01-26", name: "Republic Day", type: 'holiday' },
    { date: "2026-02-15", name: "Mahashivratri", type: 'holiday' },
    { date: "2026-03-03", name: "Holi", type: 'holiday' },
    { date: "2026-03-21", name: "Id-Ul-Fitr (Ramadan Eid)", type: 'holiday' },
    { date: "2026-03-26", name: "Shri Ram Navami", type: 'holiday' },
    { date: "2026-03-31", name: "Shri Mahavir Jayanti", type: 'holiday' },
    { date: "2026-04-03", name: "Good Friday", type: 'holiday' },
    { date: "2026-04-14", name: "Dr. Baba Saheb Ambedkar Jayanti", type: 'holiday' },
    { date: "2026-05-01", name: "Maharashtra Day", type: 'holiday' },
    { date: "2026-05-28", name: "Bakri Id", type: 'holiday' },
    { date: "2026-06-26", name: "Muharram", type: 'holiday' },
    { date: "2026-08-15", name: "Independence Day", type: 'holiday' },
    { date: "2026-09-14", name: "Ganesh Chaturthi", type: 'holiday' },
    { date: "2026-10-02", name: "Mahatma Gandhi Jayanti", type: 'holiday' },
    { date: "2026-10-20", name: "Dussehra", type: 'holiday' },
    { date: "2026-11-08", name: "Diwali Laxmi Pu01*", type: 'holiday' },
    { date: "2026-11-10", name: "Diwali-Balipratipada", type: 'holiday' },
    { date: "2026-11-24", name: "Prakash Gurpurb Sri Guru Nanak Dev", type: 'holiday' },
    { date: "2026-12-25", name: "Christmas", type: 'holiday' }
  ],
  
  extendedHours: [
    {
      date: '2026-11-08',
      name: 'Diwali Muhurat Trading',
      type: 'extended',
      hours: { start: '18:00', end: '19:15' }
    },
  ],
  
  weekendOpenings: []
};

/**
 * Check if current date is a market holiday
 */
function isHoliday(dateIST) {
  const dateStr = dateIST.toLocaleDateString('en-CA', { timeZone: 'Asia/Kolkata' });
  return marketSchedule.holidays.some(h => h.date === dateStr);
}

/**
 * Check if current date has extended hours (e.g., Muhurat trading)
 */
function getExtendedHours(dateIST) {
  const dateStr = dateIST.toLocaleDateString('en-CA', { timeZone: 'Asia/Kolkata' });
  return marketSchedule.extendedHours.find(h => h.date === dateStr) || null;
}

/**
 * Check if current date is a weekend opening
 */
function getWeekendOpening(dateIST) {
  const dateStr = dateIST.toLocaleDateString('en-CA', { timeZone: 'Asia/Kolkata' });
  return marketSchedule.weekendOpenings.find(h => h.date === dateStr) || null;
}

/**
 * Check if current time is within market hours
 */
function isWithinMarketHours(dateIST, hours) {
  const currentTime = dateIST.toLocaleTimeString('en-GB', { 
    timeZone: 'Asia/Kolkata',
    hour12: false,
    hour: '2-digit',
    minute: '2-digit'
  });

  return currentTime >= hours.start && currentTime <= hours.end;
}

/**
 * Main validation: Should the workflow run?
 */
function shouldRunWorkflow() {
  const now = new Date();
  const dayOfWeek = now.getDay(); // 0 = Sunday, 6 = Saturday

  // Check if it's a holiday
  if (isHoliday(now)) {
    console.log('❌ Market holiday - skipping workflow');
    return false;
  }

  // Check for weekend opening
  const weekendOpening = getWeekendOpening(now);
  if (weekendOpening) {
    if (isWithinMarketHours(now, weekendOpening.hours)) {
      console.log(`✅ Weekend opening: ${weekendOpening.name}`);
      return true;
    } else {
      console.log('❌ Weekend opening but outside hours - skipping');
      return false;
    }
  }

  // Check for extended hours (e.g., Muhurat trading)
  const extendedHours = getExtendedHours(now);
  if (extendedHours) {
    if (isWithinMarketHours(now, extendedHours.hours)) {
      console.log(`✅ Extended hours: ${extendedHours.name}`);
      return true;
    } else {
      console.log('❌ Extended hours day but outside hours - skipping');
      return false;
    }
  }

  // Regular weekend check (Saturday = 6, Sunday = 0)
  if (dayOfWeek === 0 || dayOfWeek === 6) {
    console.log('❌ Weekend - skipping workflow');
    return false;
  }

  // Check normal market hours (Monday-Friday)
  if (isWithinMarketHours(now, marketSchedule.normalHours)) {
    console.log('✅ Within normal market hours');
    return true;
  }

  console.log('❌ Outside market hours - skipping workflow');
  return false;
}

export default {
  async scheduled(event, env, ctx) {
    console.log('🕐 Cron triggered at:', new Date().toISOString());

    if (!shouldRunWorkflow()) {
      console.log('⏭️  Skipping workflow execution');
      return;
    }

    try {
      const response = await fetch(
        `https://api.github.com/repos/${env.OWNER}/${env.REPO}/actions/workflows/${env.WORKFLOW}/dispatches`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${env.GITHUB_TOKEN}`,
            'Accept': 'application/vnd.github.v3+json',
            'User-Agent': 'Cloudflare-Worker'
          },
          body: JSON.stringify({ ref: 'master' })
        }
      );

      if (response.ok) {
        console.log('✅ Workflow triggered successfully');
      } else {
        const error = await response.text();
        console.error('❌ Failed to trigger workflow:', error);
      }
    } catch (error) {
      console.error('❌ Error:', error);
    }
  },

  // Optional: HTTP endpoint to manually trigger or check status
  async fetch(request, env, ctx) {
    if (request.method === 'GET') {
      const canRun = shouldRunWorkflow();
      const now = new Date();
      
      return new Response(JSON.stringify({
        active: true,
        canRunNow: canRun,
        currentTime: now.toLocaleString('en-IN', { timeZone: 'Asia/Kolkata' }),
        message: canRun 
          ? 'Market is open - workflow can run' 
          : 'Market is closed - workflow will be skipped'
      }, null, 2), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Manually trigger via POST request
    const triggerEvent = { scheduledTime: Date.now() };
    await this.scheduled(triggerEvent, env, ctx);
    
    return new Response('Workflow trigger initiated', { status: 200 });
  }
};
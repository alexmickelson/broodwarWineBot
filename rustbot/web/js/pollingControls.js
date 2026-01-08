// Centralized polling control system

let pollInterval = 500; // Default 500ms
let registeredPollers = [];

export function setPollInterval(ms) {
  pollInterval = ms;
  console.log(`Poll interval set to ${ms}ms`);

  // Restart all active pollers with new interval
  registeredPollers.forEach((poller) => {
    if (poller.isActive) {
      poller.restart();
    }
  });
}

export function getPollInterval() {
  return pollInterval;
}

export function registerPoller(name, callback) {
  let intervalId = null;
  let isActive = false;

  const poller = {
    name,
    isActive: false,

    start() {
      if (isActive) return;

      console.log(`Starting poller: ${name}`);
      isActive = true;
      this.isActive = true;

      // Initial call
      callback().catch((err) => {
        console.error(`Poller ${name} initial call error:`, err);
      });

      // Set up interval
      intervalId = setInterval(async () => {
        try {
          await callback();
        } catch (err) {
          console.error(`Poller ${name} error:`, err);
        }
      }, pollInterval);
    },

    stop() {
      if (!isActive) return;

      console.log(`Stopping poller: ${name}`);
      isActive = false;
      this.isActive = false;

      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    },

    restart() {
      this.stop();
      this.start();
    },
  };

  registeredPollers.push(poller);
  return poller;
}

export function stopAll() {
  registeredPollers.forEach((poller) => poller.stop());
}

export function getActivePollers() {
  return registeredPollers.filter((p) => p.isActive).map((p) => p.name);
}

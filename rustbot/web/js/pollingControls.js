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
  const poller = {
    name,
    intervalId: null,
    isActive: false,

    start() {
      if (this.isActive) return;

      console.log(`Starting poller: ${name}`);
      this.isActive = true;

      // Initial call
      callback().catch((err) => {
        console.error(`Poller ${name} initial call error:`, err);
      });

      // Set up interval - use getPollInterval() to get current value
      this.intervalId = setInterval(async () => {
        try {
          await callback();
        } catch (err) {
          console.error(`Poller ${name} error:`, err);
        }
      }, getPollInterval());
    },

    stop() {
      if (!this.isActive) return;

      console.log(`Stopping poller: ${name}`);
      this.isActive = false;

      if (this.intervalId) {
        clearInterval(this.intervalId);
        this.intervalId = null;
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

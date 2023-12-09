
export default class WebworkerPool {
  constructor(maxSize, script) {
    this.maxSize = maxSize; // Maximum number of workers
    this.pool = []; // Pool of available workers
    this.activeWorkers = new Set(); // Set of currently active workers
    this.taskQueue = []; // Queue of pending tasks

    for (let i = 0; i < this.maxSize; i++) {
      const worker = new Worker(script);
      this.pool.push(worker);
    }
  }

  // Get a worker from the pool, or queue the task if all workers are busy
  getWorker() {
    if (this.pool.length > 0) {
      const worker = this.pool.pop();
      this.activeWorkers.add(worker);
      return worker;
    } else {
      return null; // Return null if no worker is available
    }
  }

  // Release a worker back to the pool and run the next task in the queue, if any
  releaseWorker(worker) {
    this.activeWorkers.delete(worker);
    this.pool.push(worker);

    if (this.taskQueue.length > 0) {
      // If there are queued tasks, run the next one
      const nextTask = this.taskQueue.shift();
      this.runTask(nextTask.worker, nextTask.taskData);
    }
  }

  // Run a task with a given worker
  runTask(worker, taskData) {
    worker.onmessage = (e) => {
      if (taskData.onComplete) {
        taskData.onComplete(e.data);
      }
      this.releaseWorker(worker);
    };

    worker.onerror = (err) => {
      if (taskData.onError) {
        taskData.onError(err);
      }
      this.releaseWorker(worker);
    };

    worker.postMessage(taskData.message);
  }

  // Add a new task to the pool, either run it immediately or queue it
  addTaskAsync(taskData) {
    return new Promise((resolve, reject) => {
      const worker = this.getWorker();
      if (worker) {
        this.runTask(worker, {
          ...taskData,
          onComplete: resolve,
          onError: reject
        });
      } else {
        // If no worker is available, queue the task
        this.taskQueue.push({
          worker,
          taskData: {
            ...taskData,
            onComplete: resolve,
            onError: reject
          }
        });
      }
    });
  }
}

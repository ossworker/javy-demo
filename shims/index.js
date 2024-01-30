let now = Date.now();

let log = [];

globalThis.console = {
    log: (...args) => {
        try {
            __console_log(args.map(a => JSON.stringify(a)).join(","));
            log.push({
                msSinceRun: Date.now() - now,
                lines: args.map(a => JSON.stringify(a))
            });
        } catch (e) {
            log.push({
                msSinceRun: Date.now() - now,
                lines: [JSON.stringify('failed to parse logging line')]
            });
        }
    }
};

let handlerFunction;

globalThis.__setNowDate = (nowDate) => {
    now = nowDate;
}

globalThis.__addHandler = (handlerFunc) => {
    handlerFunction = handlerFunc;
}

const requestHandler = (input) => {
    return handlerFunction(input, {moment: __GLOBAL__DAYJS, dayjs: __GLOBAL__DAYJS, Big: Big, env: __GLOBAL__ENV})
};


// This is the entrypoint for the project.
entrypoint = (input) => {
    try {
        const handlerResult = requestHandler(input);
        Promise.resolve(handlerResult)
            .then(res => {
                result = {
                    output: res,
                    log,
                };
            }).catch((err) => {
            error = `Couldn't process the response from the handler:\n${err}`;
        })
    } catch (err) {
        error = `There was an error running the handler:\n${err}\n${err.stack}`;
    }
};

// Set the result
result = null;

// Save error
error = null;
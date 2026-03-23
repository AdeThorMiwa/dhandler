const CMD = {
  OPEN_APPLICATION_MODAL: "OPEN_APPLICATION_MODAL",
  NEXT_QUESTION: "NEXT_QUESTION",
  INIT_ITERATOR: "INIT_ITERATOR",
  ANSWER_QUESTION: "ANSWER_QUESTION",
  NEXT_PAGE: "NEXT_PAGE",
};

class Scrappy {
  #iterators = null;
  #answerHandles = new Map();
  #id_counter = 0;

  constructor() {
    console.log("[scrappy] scappy initiated");
  }

  async executeCommand(cmd, payload) {
    switch (cmd) {
      case CMD.OPEN_APPLICATION_MODAL:
        return await this.#openApplicationModal(payload);
      case CMD.NEXT_QUESTION:
        return this.#nextQuestion();
      case CMD.INIT_ITERATOR:
        return this.#initIterator();
      case CMD.ANSWER_QUESTION:
        return this.#answerQuestion(payload);
      case CMD.NEXT_PAGE:
        return await this.#nextPage();
      default:
        console.log("[scrappy] unknown command");
    }
  }

  async #openApplicationModal(payload) {
    console.log("[scrappy] opening application modal");
    const jobId = payload.jobId;
    const easyApplyButton = await this.#waitForElement(
      `a[href*='/jobs/view/${jobId}/apply']`,
    );
    easyApplyButton.click();

    const interopOutlet = await this.#waitForElement("#interop-outlet");

    await new Promise((resolve, reject) => {
      const shadowRoot = interopOutlet.shadowRoot;

      // Check if already rendered
      if (shadowRoot.querySelector(".fb-dash-form-element")) {
        return resolve();
      }

      const timer = setTimeout(() => {
        observer.disconnect();
        reject(new Error("Timed out waiting for modal content"));
      }, 20000);

      const observer = new MutationObserver(() => {
        if (shadowRoot.querySelector(".fb-dash-form-element")) {
          clearTimeout(timer);
          observer.disconnect();
          resolve();
        }
      });

      // Observe the shadow root directly, not document.body
      observer.observe(shadowRoot, {
        childList: true,
        subtree: true,
      });
    });

    console.log("[scrappy] modal opened", Date.now());
  }

  #initIterator() {
    console.log("[scrappy] creating question iterator for page");
    let interop = document.querySelector("#interop-outlet");
    let questions = interop.shadowRoot.querySelectorAll(
      ".fb-dash-form-element, .js-jobs-document-upload__container",
    );
    console.log("questions: ", questions);
    let questionBlockIterator = questions[Symbol.iterator]();
    this.#iterators = questionBlockIterator;
  }

  #nextQuestion() {
    console.log("[scrappy] getting next question");
    let nextItem = this.#iterators.next();
    console.log("[scrappy] next: ", nextItem);
    if (nextItem.done) {
      this.#iterators = null;
      return null;
    }
    let question = this.#createQuestion(nextItem.value);
    return question;
  }

  #createQuestion(block) {
    let input = block.querySelector(
      "input[type='text'], input[type='email'], input[type='number']",
    );
    if (input) {
      return this.#extractInputQuestion(block, input);
    }

    let select = block.querySelector(
      "select[data-test-text-entity-list-form-select]",
    );
    if (select) {
      return this.#extractSelectQuestion(block, select);
    }

    // type ahead
    let typeAhead = block.querySelector(
      "[data-test-single-typeahead-entity-form-component] input",
    );
    if (typeAhead) {
      return this.#extractTypeaheadQuestion(block, typeAhead);
    }

    // file upload
    let fileUpload = block.querySelector("input[type='file']");
    if (fileUpload) {
      return this.#extractFileUploadQuestion(block);
    }

    // radios
    const radios = [...block.querySelectorAll("input[type='radio']")];
    if (radios.length > 0) {
      return this.#extractRadioQuestion(block, radios);
    }

    const checkboxes = [...block.querySelectorAll("input[type='checkbox']")];
    if (checkboxes.length > 0) {
      return this.#extractCheckboxQuestion(block, checkboxes);
    }

    return null;
  }

  #extractInputQuestion(block, input) {
    const label = this.#labelText(block);
    const required = !!block.querySelector("input[required]");
    const hint = input.getAttribute("placeholder") ?? null;
    const currentValue = input.value || null;
    const kind =
      input.getAttribute("inputmode") === "numeric" ? "number" : "text";

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "text" && answer.kind !== "number") {
        return;
      }

      // Clear existing value
      input.focus();
      input.value = "";
      // Trigger input event so React/Vue picks up the change
      input.dispatchEvent(new Event("input", { bubbles: true }));
      // Set new value
      input.value = String(answer.value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
      input.blur();
    });

    return {
      id,
      kind,
      label,
      required,
      hint,
      current_value: currentValue,
      options: [],
    };
  }

  #extractSelectQuestion(block, select) {
    const label = this.#labelText(block);
    const required =
      select.required || select.getAttribute("aria-required") === "true";
    const options = [...select.options]
      .map((o) => ({ text: o.text.trim(), value: o.value }))
      .filter((t) => t.text !== "Select an option" && t.text !== "");
    const currentValue =
      select.value !== "Select an option" ? select.value : null;

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "dropdown") {
        return;
      }

      select.focus();
      select.value = answer.value;
      select.dispatchEvent(new Event("change", { bubbles: true }));
      select.dispatchEvent(new Event("input", { bubbles: true }));
      select.blur();
    });

    return {
      id,
      kind: "dropdown",
      label,
      required,
      hint: null,
      current_value: currentValue,
      options,
    };
  }

  #extractTypeaheadQuestion(block, typeahead) {
    const label = this.#labelText(block);
    const required = !!block.querySelector("input[required]");
    const currentValue = typeahead.value || null;
    const hint = typeahead.getAttribute("placeholder") ?? null;

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "text") {
        return;
      }

      typeahead.focus();
      typeahead.value = "";
      typeahead.dispatchEvent(new Event("input", { bubbles: true }));
      typeahead.value = answer.value;
      typeahead.dispatchEvent(new Event("input", { bubbles: true }));
      // Typeahead usually shows a dropdown after input — wait for it
      // and pick the first suggestion
      setTimeout(() => {
        const suggestion = document.querySelector(
          "[data-test-typeahead-item], [role='option']",
        );
        if (suggestion) {
          suggestion.click();
        } else {
          // No suggestion, just blur to accept raw value
          typeahead.dispatchEvent(new Event("change", { bubbles: true }));
          typeahead.blur();
        }
      }, 300);
    });

    return {
      id,
      kind: "text",
      label,
      required,
      hint,
      current_value: currentValue,
      options: [],
    };
  }

  #extractFileUploadQuestion(block) {
    const fileInput = block.querySelector("input[type='file']");
    const labelEl = block.querySelector("label span[role='button']");

    // Extract label from aria-label or span text
    const label =
      labelEl?.getAttribute("aria-label") ??
      labelEl?.innerText?.trim() ??
      "File upload";

    // Extract accepted formats from accept attribute
    const accept = fileInput?.getAttribute("accept") ?? "";
    const hint =
      accept
        .split(",")
        .map((t) => t.trim().split("/").pop().split(".").pop())
        .join(", ") || null;

    const required = fileInput?.hasAttribute("required") ?? false;

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "file_upload") {
        return;
      }

      // Convert base64 bytes to a File object and assign via DataTransfer
      const { filename, base64 } = answer.value;
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }

      // Determine mime type from filename
      const mimeType = filename.endsWith(".pdf")
        ? "application/pdf"
        : "application/octet-stream";

      // Create File object and assign via DataTransfer
      const file = new File([bytes], filename, { type: mimeType });
      const dt = new DataTransfer();
      dt.items.add(file);
      fileInput.files = dt.files;

      // Trigger change so LinkedIn registers the upload
      fileInput.dispatchEvent(new Event("change", { bubbles: true }));
      fileInput.dispatchEvent(new Event("input", { bubbles: true }));
    });

    return {
      id,
      kind: "file_upload",
      label,
      required,
      hint,
      current_value: null,
      options: [],
    };
  }

  #extractRadioQuestion(block, radios) {
    const label = this.#labelText(block);
    const options = radios.map((r) => ({
      text: r.getAttribute("aria-label") ?? r.value,
      value: r.value,
    }));
    const checked = radios.find((r) => r.checked);
    const currentValue = checked?.value ?? null;
    const isYesNo =
      options.length === 2 &&
      options.some((o) => o.value.toLowerCase().includes("yes")) &&
      options.some((o) => o.value.toLowerCase().includes("no"));
    const required = !!block.querySelector("input[aria-required='true']");

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "yes_no" && answer.kind !== "single_choice") {
        return;
      }

      const value =
        answer.kind === "yes_no" ? (answer.value ? "Yes" : "No") : answer.value;

      let labels = Array.from(block.querySelectorAll("label"));

      const target = labels.find(
        (l) =>
          l
            .getAttribute("data-test-text-selectable-option__label")
            .toLowerCase() == value.toLowerCase(),
      );

      target?.click();
    });

    return {
      id,
      kind: isYesNo ? "yes_no" : "single_choice",
      label,
      required,
      hint: null,
      current_value: currentValue,
      options,
    };
  }

  #extractCheckboxQuestion(block, checkboxes) {
    const label = this.#labelText(block);
    const options = checkboxes.map(
      (c) => c.getAttribute("aria-label") ?? c.value,
    );

    let id = this.get_next_id();
    this.#answerHandles.set(id, (answer) => {
      if (["skip", "missing_required_info"].includes(answer.kind)) return;

      if (answer.kind !== "multi_choice") {
        return;
      }

      const chosen = new Set(answer.value);

      checkboxes.forEach((cb) => {
        const label = cb.getAttribute("aria-label") ?? cb.value;
        const shouldBeChecked = chosen.has(label);

        if (cb.checked !== shouldBeChecked) {
          cb.focus();
          cb.checked = shouldBeChecked;
          cb.dispatchEvent(new Event("change", { bubbles: true }));
          cb.dispatchEvent(new Event("click", { bubbles: true }));
          cb.blur();
        }
      });
    });

    return {
      id,
      kind: "multi_choice",
      label,
      required: false,
      hint: null,
      current_value: null,
      options,
    };
  }

  #labelText(el) {
    const titleSpan = el.querySelector(
      "[data-test-text-entity-list-form-title] span[aria-hidden='true']",
    );
    if (titleSpan?.innerText?.trim()) {
      return titleSpan.innerText.trim();
    }

    const label = el.querySelector("label, legend");
    if (label?.innerText?.trim()) {
      return label.innerText.trim();
    }

    return "";
  }

  #answerQuestion(payload) {
    console.log("payload: ", payload);
    let handler = this.#answerHandles.get(payload.handle);
    handler(payload.answer);
  }

  async #nextPage() {
    const interopOutlet = document.querySelector("#interop-outlet");
    const shadowRoot = interopOutlet.shadowRoot;

    let nextButton = shadowRoot.querySelector(
      "button[aria-label='Continue to next step']",
    );

    if (!nextButton) {
      nextButton = shadowRoot.querySelector(
        "button[aria-label='Review your application']",
      );
    }

    if (!nextButton) {
      return false;
    }

    const progressEl = shadowRoot.querySelector(
      ".artdeco-completeness-meter-linear__progress-element",
    );
    const currentProgress = progressEl
      ? parseInt(progressEl.getAttribute("aria-valuenow") ?? "0")
      : 0;

    nextButton.click();

    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        observer.disconnect();
        reject(new Error("Timed out waiting for next page"));
      }, 10000);

      const observer = new MutationObserver(() => {
        const newProgressEl = shadowRoot.querySelector(
          ".artdeco-completeness-meter-linear__progress-element",
        );
        const newProgress = newProgressEl
          ? parseInt(newProgressEl.getAttribute("aria-valuenow") ?? "0")
          : 0;

        if (newProgress > currentProgress) {
          clearTimeout(timer);
          observer.disconnect();
          resolve();
        }
      });

      observer.observe(shadowRoot, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["aria-valuenow", "value"],
      });
    });

    return true;
  }

  #waitForElement(selector, timeout = 10000) {
    return new Promise((resolve, reject) => {
      const existing = document.querySelector(selector);
      if (existing) return resolve(existing);

      const timer = setTimeout(() => {
        observer.disconnect();
        reject(new Error(`Timed out waiting for ${selector}`));
      }, timeout);

      const observer = new MutationObserver(() => {
        const el = document.querySelector(selector);
        if (el) {
          clearTimeout(timer);
          observer.disconnect();
          resolve(el);
        }
      });

      observer.observe(document.body, { childList: true, subtree: true });
    });
  }

  get_next_id() {
    const counter = this.#id_counter++;
    return `id-${counter}`;
  }
}

window.scrappy = new Scrappy();

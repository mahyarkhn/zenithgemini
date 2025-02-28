# ZenithGemini - Google Gemini Integration for Telegram

**A robust and efficient Telegram bot facilitating seamless interaction with Google's Gemini AI.**

ZenithGemini provides a dedicated interface for accessing Google Gemini's advanced capabilities directly within the Telegram environment. This project focuses on delivering a reliable and user-friendly experience, enabling users to leverage cutting-edge AI for information retrieval and task assistance.

## Key Features

* **Direct Google Gemini API Integration:** Enables real-time access to Google Gemini's generative AI models.
* **Comprehensive Language Support:** Facilitates interaction in all languages supported by Google Gemini.
* **Inline Query Functionality:** Provides instant responses directly within any Telegram chat.
* **MarkdownV2 Formatting:** Ensures clear and aesthetically pleasing response presentation.
* **Explicit Query Termination:** Utilizes the "!!" identifier for precise query submission in inline mode.

## Getting Started

### Prerequisites

* Telegram Account
* Google Cloud Account
* Rust (Latest Stable Release)

### Deployment

1.  **Telegram Bot Creation:**
    * Initiate a new bot via [BotFather](https://t.me/botfather) using the `/newbot` command.
    * Retrieve the generated bot token.

2.  **Google Gemini API Key Acquisition:**
    * Generate an API key through [Google AI Studio](https://aistudio.google.com/app/apikey).

3.  **Repository Cloning:**

    ```bash
    git clone https://github.com/mahyarkhn/zenithgemini --depth=1
    cd zenithgemini
    ```

4.  **Environment Variable Configuration:**
    * Create a `.env` file within the `zenithgemini` directory.
    * Populate the `.env` file with the following configurations:

        ```env
        RUST_LOG=info
        TELOXIDE_TOKEN=YOUR_BOT_TOKEN
        GEMINI_API_KEY=YOUR_GEMINI_API_KEY
        ```

5.  **Bot Execution:**

    ```bash
    cargo run --release
    ```

## Usage Guidelines

* **Direct Message Queries:** Employ the `/generate <query>` command within a direct chat with the bot. Example: `/generate What is the capital of France?`
* **Inline Query Utilization:** Input `@your_bot_username <query> !!` within any Telegram chat.
* **Query Termination Signal:** Utilize "!!" to explicitly signify the end of an inline query.

## Example Interaction

User: `/generate What are the primary export goods of Japan?`<br>
Bot: `Japan's primary export goods include automobiles, electronics, and machinery.`


## Contribution

Contributions aimed at enhancing functionality, improving performance, or addressing identified issues are encouraged. Please submit pull requests or open issues for review.

## License

This project is distributed under the [MIT License](LICENSE).

## Acknowledgments

* [Teloxide](https://github.com/teloxide/teloxide): For the robust Telegram bot framework.
* [Google Gemini](https://ai.google.dev/): For the advanced generative AI capabilities.
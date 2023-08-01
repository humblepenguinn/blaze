# Usage

## `init` - Initialize a New Node.js Project
To create a new Node.js project, use the `init` command. This command will prompt you for various options to configure your project, and then it will set up the necessary files and directories.

```bash
blaze init
```

Simply follow the prompts to set up your project. Once completed, you will have a new Node.js project ready for development.

## `install` - Install Node.js Packages
The `install` command allows you to install Node.js packages into your project. You can provide one or more package names as arguments to install them directly.

```bash
blaze install <package_name1> <package_name2> ...
```

For example, to install the lodash and express packages, you would run:

```bash
blaze install lodash express
```

If you don't provide any package names, `Blaze` will look for a package.json file in your project directory and download the dependencies listed there.

```bash
blaze install
```

## `help` - Get Help
If you ever need assistance or want to explore available commands, you can use the `help` command. It will provide you with information about the commands and their usage.

```bash
blaze help
```

## `version` - Check Blaze Version
To check the version of Blaze installed on your system, use the `version` command:

```bash
blaze version
```
This will display the version of Blaze.

You can also pass in `true` as an argument to enable verbose output:

```bash
blaze version true
```
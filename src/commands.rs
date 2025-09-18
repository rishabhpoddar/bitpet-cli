use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    // Get the mood, health, and other details of your pet
    Status {},

    /// Feed your pet (based on your git commits since last feed)
    Feed {},

    /// Play with your pet (Makes it happy)
    Play {},

    /// Add a git repo (will be used to fetch commits for feeding your pet)
    AddRepo {
        path: String,
    },

    /// Remove a git repo (will not be used to fetch commits for feeding your pet)
    RemoveRepo {
        path: String,
    },

    /// List all the git repos from which commits will be fetched for feeding your pet
    ListRepos {},

    /// Login to your BitPet account
    Login {},

    /// Logout from your BitPet account
    Logout {},
    /// Get the user information about who is logged in
    Whoami {},

    /// Adopt a new pet if you don't already have one.
    NewPet {},

    /// Let go of your pet
    RemovePet {},
}

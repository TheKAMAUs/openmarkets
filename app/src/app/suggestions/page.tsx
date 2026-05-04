import { SuggestionAPI } from "@/utils/interactions/dataGetter";
import SuggestionsClient from "./_components/SuggestionsClient";


// const SuggestionsPage = async () => {
//     // Fetch suggestions on the server (public endpoint)
//     const suggestions = await SuggestionAPI.getPublicSuggestions();

//     return <SuggestionsClient initialSuggestions={suggestions} />;
// };

// export default SuggestionsPage;



const SuggestionsPage = () => {
    return <SuggestionsClient initialSuggestions={[]} />;
};

export default SuggestionsPage;
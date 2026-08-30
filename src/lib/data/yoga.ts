export interface YogaPose {
  id: string;
  number: number;
  name: string;
  sanskritName: string;
  category: 'Resting' | 'Strength' | 'Standing Balance' | 'Backbend' | 'Forward Bend' | 'Restorative';
  defaultDurationSec: number;
  minDurationSec: number;
  maxDurationSec: number;
  position: string;
  description: string;
  benefits: string;
  spokenGuidance: string;
  halfwayCue: string;
  accentColor: string; // Tailwind color name like 'emerald', 'cyan', 'purple', 'amber', 'rose'
}

export const YOGA_POSES: YogaPose[] = [
  {
    id: 'childs-pose',
    number: 1,
    name: "Child’s Pose",
    sanskritName: 'Balasana',
    category: 'Resting',
    defaultDurationSec: 90,
    minDurationSec: 60,
    maxDurationSec: 180,
    position: 'Kneeling, forward resting position.',
    description:
      'Kneel on the floor, touch your big toes together, and widen your knees slightly wider than your hips. Exhale and lay your torso down between your thighs. Extend your arms straight in front of you with palms facing down, and rest your forehead gently on the mat.',
    benefits:
      'This is a foundational resting pose. It gently stretches the hips, thighs, and ankles while relieving back and neck tension. Use this anytime you need a break during a longer routine.',
    spokenGuidance:
      "Welcome to Child's Pose, Balasana. Kneel comfortably, touch your big toes together, and widen your knees slightly wider than your hips. Exhale and extend your torso down between your thighs. Reach your arms forward, rest your forehead gently on the mat, and allow your lower back to release with every breath.",
    halfwayCue: 'Halfway through. Continue slow, steady breathing, softening your chest and shoulders closer to the ground.',
    accentColor: 'emerald',
  },
  {
    id: 'downward-dog',
    number: 2,
    name: 'Downward-Facing Dog',
    sanskritName: 'Adho Mukha Svanasana',
    category: 'Strength',
    defaultDurationSec: 45,
    minDurationSec: 30,
    maxDurationSec: 90,
    position: 'Inverted V-shape.',
    description:
      'Start on your hands and knees in tabletop position. Tuck your toes under, lift your knees off the floor, and push your pelvis up toward the ceiling. Keep your hands shoulder-width apart and press your chest back toward your thighs. If your hamstrings are tight, keep a slight bend in your knees.',
    benefits:
      'One of the most famous poses in yoga, it builds upper body strength, stretches the hamstrings and calves, and gets blood flowing to the brain.',
    spokenGuidance:
      'Transitioning into Downward-Facing Dog, Adho Mukha Svanasana. Tuck your toes and lift your hips high toward the sky to form an inverted V-shape. Spread your fingers wide, press firmly through your palms, and draw your chest back toward your thighs. Keep a gentle bend in your knees if needed.',
    halfwayCue: 'Hold steady and breathe. Push through your hands, lengthen your spine, and let your head hang freely between your arms.',
    accentColor: 'cyan',
  },
  {
    id: 'warrior-2',
    number: 3,
    name: 'Warrior II',
    sanskritName: 'Virabhadrasana II',
    category: 'Strength',
    defaultDurationSec: 60,
    minDurationSec: 30,
    maxDurationSec: 120,
    position: 'Standing lunge with open hips.',
    description:
      'Step your feet wide apart (about 3-4 feet). Turn your right foot out 90 degrees and angle your left foot slightly inward. Bend your right knee so it tracks directly over your right ankle. Extend your arms out to the sides, parallel to the floor, and gaze fiercely over your right fingertips.',
    benefits:
      'Warrior poses are all about building stamina and heat. It strengthens the quads and glutes while opening the hips and chest.',
    spokenGuidance:
      'Step wide into Warrior Two, Virabhadrasana Two. Turn your front foot forward and bend into your front knee at ninety degrees. Ground your back foot firmly, extend your arms wide and parallel to the floor, and focus your gaze over your front fingertips with power and composure.',
    halfwayCue: 'Switch sides now. Bend into the opposite knee, extend both arms parallel to the floor, and root through your heels.',
    accentColor: 'amber',
  },
  {
    id: 'tree-pose',
    number: 4,
    name: 'Tree Pose',
    sanskritName: 'Vrksasana',
    category: 'Standing Balance',
    defaultDurationSec: 60,
    minDurationSec: 30,
    maxDurationSec: 120,
    position: 'Standing balance.',
    description:
      'Stand tall with your feet together. Shift your weight onto your left leg. Lift your right foot and place the sole against your inner left thigh. (If this is too difficult, place it on your inner calf—just avoid putting pressure directly on the knee joint). Bring your hands together in front of your chest. Find a fixed point to stare at to help you balance.',
    benefits:
      'Tree pose improves focus, balance, and mind-body awareness while strengthening the stabilizing muscles in your ankles and feet.',
    spokenGuidance:
      'Stand tall for Tree Pose, Vrksasana. Root down through your standing leg. Bring the sole of your opposite foot against your inner calf or thigh. Bring your hands together in prayer at your chest, fix your gaze on a steady focal point, and balance with calm intention.',
    halfwayCue: 'Switch standing legs now. Shift your weight smoothly, place the sole against your inner leg, and find your center.',
    accentColor: 'emerald',
  },
  {
    id: 'cobra-pose',
    number: 5,
    name: 'Cobra Pose',
    sanskritName: 'Bhujangasana',
    category: 'Backbend',
    defaultDurationSec: 30,
    minDurationSec: 15,
    maxDurationSec: 60,
    position: 'Prone (face-down) backbend.',
    description:
      'Lie on your stomach with your legs straight back and the tops of your feet flat on the floor. Place your hands flat on the mat directly under your shoulders and hug your elbows close to your ribs. Inhale and gently press through your hands to lift your chest off the floor. Keep your neck long and your gaze slightly forward.',
    benefits:
      'Cobra counteracts the forward-slouching posture most of us have from sitting at desks or looking at phones. It strengthens the spine and opens the chest.',
    spokenGuidance:
      'Lower onto your belly for Cobra Pose, Bhujangasana. Tops of your feet flat, hands placed directly under your shoulders with elbows hugging your ribs. Inhale and press gently through your hands to lift your chest and open your heart forward. Keep your shoulders down and neck long.',
    halfwayCue: 'Maintain a gentle curve in your spine. Keep your glutes engaged and breathe smoothly into your ribcage.',
    accentColor: 'rose',
  },
  {
    id: 'seated-forward-fold',
    number: 6,
    name: 'Seated Forward Fold',
    sanskritName: 'Paschimottanasana',
    category: 'Forward Bend',
    defaultDurationSec: 90,
    minDurationSec: 60,
    maxDurationSec: 180,
    position: 'Seated forward bend.',
    description:
      'Sit on the floor with your legs extended straight in front of you. Inhale to lengthen your spine and sit up tall. Exhale and hinge from your hips (not your lower back) to fold forward over your legs. Rest your hands on your shins, ankles, or feet.',
    benefits:
      'A deeply calming pose that stretches the entire back of the body, from the heels up through the hamstrings, spine, and neck. It is known to help relieve stress and mild depression.',
    spokenGuidance:
      'Transition to a seated posture for Seated Forward Fold, Paschimottanasana. Extend your legs straight ahead. Inhale to lengthen your spine, and exhale to hinge forward from your hips. Let your hands rest naturally on your shins, ankles, or toes without straining.',
    halfwayCue: 'Allow each exhalation to soften your spine further forward, releasing any tension in your hamstrings and lower back.',
    accentColor: 'cyan',
  },
  {
    id: 'bridge-pose',
    number: 7,
    name: 'Bridge Pose',
    sanskritName: 'Setu Bandhasana',
    category: 'Backbend',
    defaultDurationSec: 45,
    minDurationSec: 30,
    maxDurationSec: 90,
    position: 'Supine (face-up) backbend.',
    description:
      'Lie on your back with your knees bent and feet flat on the floor, hip-width apart. Walk your heels in close to your glutes. Keep your arms alongside your body. Press your feet into the floor, exhale, and push your hips up toward the ceiling.',
    benefits:
      'Bridge pose opens up the front of the body, stretching the chest and neck while stimulating the thyroid and abdominal organs. It also strengthens the glutes and lower back.',
    spokenGuidance:
      'Lie on your back for Bridge Pose, Setu Bandhasana. Bend your knees, plant your feet hip-width apart near your hips, and rest your arms by your sides. Exhale, press firmly into your feet, and lift your hips toward the ceiling. Expand your chest toward your chin.',
    halfwayCue: 'Keep your knees aligned over your ankles, glutes active, and thighs parallel. Breathe deeply into your open chest.',
    accentColor: 'purple',
  },
  {
    id: 'corpse-pose',
    number: 8,
    name: 'Corpse Pose',
    sanskritName: 'Savasana',
    category: 'Restorative',
    defaultDurationSec: 180,
    minDurationSec: 60,
    maxDurationSec: 600,
    position: 'Lying flat on the back.',
    description:
      'Lie completely flat on your back. Separate your legs slightly and let your feet splay open naturally. Rest your arms comfortably by your sides with your palms facing up. Close your eyes, let go of any controlled breathing, and consciously relax every single muscle in your body starting from your toes up to your jaw.',
    benefits:
      'Savasana is traditionally the final resting pose of any yoga sequence. It allows your body to physically absorb the benefits of the work you just did and completely resets the nervous system.',
    spokenGuidance:
      'Finally, release into Corpse Pose, Savasana. Lie completely flat on your back. Let your legs separate and feet splay open naturally. Rest your arms by your sides with palms turned toward the sky. Close your eyes, let go of all effort, and surrender your body to total tranquility.',
    halfwayCue: 'Sink deeper into stillness. Release all muscular tension from your face, shoulders, hips, and limbs. Simply be.',
    accentColor: 'purple',
  },
];
